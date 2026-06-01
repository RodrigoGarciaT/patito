use crate::types::{Type, TEMP_INT_BASE, TEMP_FLOAT_BASE};
use crate::semantic_cube::{Op, result_type, is_assignable};

// ── Operando ──────────────────────────────────────────────────────────────────
//
// Representa una entrada en la pila de operandos. Combina la dirección (que
// aparecerá en los cuádruplos) con su tipo (necesario para consultar el
// cubo semántico al colapsar).
//
// La dirección puede ser:
//   · de una variable global o local      (resuelta vía func_dir)
//   · de una constante literal             (resuelta vía const_table)
//   · de un temporal generado              (asignada por new_temp)
//
// Entrega 4: antes el campo era `name: String` y guardaba nombres como "x",
// "5" o "t1". Ahora guarda la dirección de memoria virtual correspondiente.

#[derive(Debug, Clone)]
pub struct Operand {
    pub address: u32,
    pub ty:      Type,
}

// ── Cuádruplo ─────────────────────────────────────────────────────────────────
//
// Representación intermedia clásica de cuatro campos: (op, arg1, arg2, result).
// Los campos siguen siendo `String` (las direcciones se imprimen como números).
// Los campos vacíos se rellenan con "_". Para saltos pendientes (GOTOF, GOTO)
// el campo result se inicializa con "___" y se rellena después con el índice
// del cuádruplo destino (técnica de back-patching).

#[derive(Debug, Clone)]
pub struct Quadruple {
    pub op:     Op,
    pub arg1:   String,
    pub arg2:   String,
    pub result: String,
}

impl Quadruple {
    pub fn new(op: Op, arg1: impl Into<String>, arg2: impl Into<String>, result: impl Into<String>) -> Self {
        Quadruple { op, arg1: arg1.into(), arg2: arg2.into(), result: result.into() }
    }
}

impl std::fmt::Display for Quadruple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:<9}, {:<6}, {:<6}, {:<6})", format!("{}", self.op), self.arg1, self.arg2, self.result)
    }
}

// ── Marcador de salto pendiente ───────────────────────────────────────────────

const PENDING: &str = "___";

// ── Generador de cuádruplos ───────────────────────────────────────────────────
//
// Encapsula toda la maquinaria del método clásico de Patito:
//
//   · operands     — pila de operandos con su tipo y dirección
//   · operators    — pila de operadores
//   · jumps        — pila de índices a cuádruplos pendientes de back-patching
//   · quads        — la fila de cuádruplos emitidos
//   · n_temp_int   — contador de temporales enteros usados (per-función)
//   · n_temp_float — contador de temporales flotantes usados (per-función)
//
// QuadGen NO conoce el scope ni la tabla de variables. Los métodos que pueden
// fallar por tipos devuelven Result<…, String> en lugar de acumular errores;
// el llamador (SemanticContext) se encarga de capturarlos.
//
// Entrega 4: el antiguo `temp_counter` único que producía "t1", "t2"... fue
// reemplazado por contadores per-tipo que se traducen a direcciones reales en
// los segmentos TEMP_INT_BASE / TEMP_FLOAT_BASE. Los contadores se reinician al
// entrar a cada función porque las temporales viven en el activation record.

pub struct QuadGen {
    pub operands:     Vec<Operand>,
    pub operators:    Vec<Op>,
    pub jumps:        Vec<usize>,
    pub quads:        Vec<Quadruple>,
    n_temp_int:       u32,
    n_temp_float:     u32,
}

impl Default for QuadGen {
    fn default() -> Self {
        QuadGen {
            operands:     Vec::new(),
            operators:    Vec::new(),
            jumps:        Vec::new(),
            quads:        Vec::new(),
            n_temp_int:   0,
            n_temp_float: 0,
        }
    }
}

impl QuadGen {
    pub fn new() -> Self { Self::default() }

    // ── Helpers internos ──────────────────────────────────────────────────────

    /// Reserva la siguiente dirección temporal del tipo solicitado.
    fn new_temp(&mut self, ty: Type) -> u32 {
        match ty {
            Type::Entero => {
                let a = TEMP_INT_BASE + self.n_temp_int;
                self.n_temp_int += 1;
                a
            }
            Type::Flotante => {
                let a = TEMP_FLOAT_BASE + self.n_temp_float;
                self.n_temp_float += 1;
                a
            }
            Type::Nula => unreachable!("temporal de tipo nula"),
        }
    }

    fn fill_jump(&mut self, idx: usize, target: usize) {
        if let Some(q) = self.quads.get_mut(idx) {
            q.result = target.to_string();
        }
    }

    // ── Manejo de contadores per-función ──────────────────────────────────────
    //
    // SemanticContext invoca estos al entrar y salir de cada función:
    //   reset_temp_counters() en begin_func
    //   take_temp_counts()    en end_func (para guardar el snapshot en FuncInfo)

    pub fn reset_temp_counters(&mut self) {
        self.n_temp_int = 0;
        self.n_temp_float = 0;
    }

    pub fn take_temp_counts(&self) -> (u32, u32) {
        (self.n_temp_int, self.n_temp_float)
    }

    // ── Operandos ─────────────────────────────────────────────────────────────

    /// El llamador ya resolvió tipo y dirección (consultando func_dir o ConstTable).
    pub fn push_operand(&mut self, operand: Operand) {
        self.operands.push(operand);
    }

    // ── Operadores ────────────────────────────────────────────────────────────

    /// Algoritmo de pila con precedencia:
    /// colapsa mientras top tenga precedencia >= la del nuevo operador,
    /// excepto cuando top es LParen (fondo falso, nunca se colapsa por prec).
    pub fn push_operator(&mut self, op: Op) -> Result<(), String> {
        while let Some(&top) = self.operators.last() {
            if top == Op::LParen { break; }
            if top.precedence() < op.precedence() { break; }
            self.collapse_top()?;
        }
        self.operators.push(op);
        Ok(())
    }

    // ── Paréntesis ────────────────────────────────────────────────────────────

    pub fn push_lparen(&mut self) {
        self.operators.push(Op::LParen);
    }

    /// Colapsa todo hasta encontrar el LParen, y lo pop.
    pub fn pop_lparen(&mut self) -> Result<(), String> {
        while let Some(&top) = self.operators.last() {
            if top == Op::LParen {
                self.operators.pop();
                return Ok(());
            }
            self.collapse_top()?;
        }
        Err("Paréntesis derecho sin paréntesis izquierdo correspondiente".into())
    }

    // ── Cierre de expresión completa ─────────────────────────────────────────

    /// Colapsa todo lo que haya en la pila hasta encontrar un LParen o vaciarla.
    /// Se llama al final de un statement (;) o al cerrar la cond de un control.
    pub fn force_collapse(&mut self) -> Result<(), String> {
        while let Some(&top) = self.operators.last() {
            if top == Op::LParen { break; }
            self.collapse_top()?;
        }
        Ok(())
    }

    // ── Colapso (interno) ─────────────────────────────────────────────────────

    fn collapse_top(&mut self) -> Result<(), String> {
        let op = self.operators.pop()
            .ok_or_else(|| "Pila de operadores vacía al colapsar".to_string())?;
        match op {
            Op::Asig => self.collapse_assignment(),
            _        => self.collapse_binary(op),
        }
    }

    fn collapse_binary(&mut self, op: Op) -> Result<(), String> {
        let right = self.operands.pop()
            .ok_or_else(|| format!("Falta operando derecho para {}", op))?;
        let left  = self.operands.pop()
            .ok_or_else(|| format!("Falta operando izquierdo para {}", op))?;

        match result_type(left.ty, op, right.ty) {
            None => Err(format!(
                "Tipos incompatibles: {} {} {}", left.ty, op, right.ty
            )),
            Some(ty) => {
                let t_addr = self.new_temp(ty);
                self.quads.push(Quadruple::new(
                    op,
                    left.address.to_string(),
                    right.address.to_string(),
                    t_addr.to_string(),
                ));
                self.operands.push(Operand { address: t_addr, ty });
                Ok(())
            }
        }
    }

    fn collapse_assignment(&mut self) -> Result<(), String> {
        let value = self.operands.pop()
            .ok_or_else(|| "Falta valor en asignación".to_string())?;
        let dest  = self.operands.pop()
            .ok_or_else(|| "Falta destino en asignación".to_string())?;

        if !is_assignable(dest.ty, value.ty) {
            return Err(format!(
                "Asignación incompatible: {} = {}", dest.ty, value.ty
            ));
        }
        self.quads.push(Quadruple::new(
            Op::Asig,
            value.address.to_string(),
            "_",
            dest.address.to_string(),
        ));
        // La asignación no produce valor: no se empuja resultado.
        Ok(())
    }

    // ── Statements lineales ───────────────────────────────────────────────────

    pub fn end_assignment(&mut self) -> Result<(), String> {
        // Al cerrar con ';', force-collapse vacía la pila hasta el Asig que
        // quedó en el fondo desde AsignEq, y collapse_assignment lo procesa.
        self.force_collapse()
    }

    pub fn emit_print_expr(&mut self) -> Result<(), String> {
        self.force_collapse()?;
        let operand = self.operands.pop()
            .ok_or_else(|| "Falta operando para escribe".to_string())?;
        self.quads.push(Quadruple::new(
            Op::Print, operand.address.to_string(), "_", "_"
        ));
        Ok(())
    }

    pub fn emit_print_str(&mut self, s: String) {
        self.quads.push(Quadruple::new(Op::PrintStr, s, "_", "_"));
    }

    /// Devuelve el Operand consumido para que el llamador valide su tipo
    /// contra el return_type esperado de la función actual.
    pub fn emit_return(&mut self) -> Result<Operand, String> {
        self.force_collapse()?;
        let operand = self.operands.pop()
            .ok_or_else(|| "Falta valor en regresa".to_string())?;
        self.quads.push(Quadruple::new(
            Op::Return, operand.address.to_string(), "_", "_"
        ));
        Ok(operand)
    }

    // ── Control de flujo (jumps) ──────────────────────────────────────────────

    /// Genera GOTOF con destino pendiente y empuja su índice a la pila de jumps.
    /// Devuelve el tipo de la condición para que el llamador valide que es Entero.
    pub fn gen_gotof(&mut self) -> Result<Type, String> {
        self.force_collapse()?;
        let cond = self.operands.pop()
            .ok_or_else(|| "Falta condición".to_string())?;
        self.quads.push(Quadruple::new(
            Op::GotoF, cond.address.to_string(), "_", PENDING
        ));
        self.jumps.push(self.quads.len() - 1);
        Ok(cond.ty)
    }

    /// Genera GOTO pendiente (para saltar el cuerpo del else),
    /// rellena el GOTOF anterior con la posición justo después del GOTO,
    /// y empuja el nuevo índice del GOTO.
    pub fn gen_else_goto(&mut self) -> Result<(), String> {
        let gotof_idx = self.jumps.pop()
            .ok_or_else(|| "Pila de jumps vacía en sino".to_string())?;
        self.quads.push(Quadruple::new(Op::Goto, "_", "_", PENDING));
        let goto_idx = self.quads.len() - 1;
        self.fill_jump(gotof_idx, self.quads.len()); // brinca al inicio del else
        self.jumps.push(goto_idx);
        Ok(())
    }

    /// Cierra un si (con o sin sino): rellena el salto pendiente
    /// (GOTOF o GOTO) con el índice del cuádruplo siguiente.
    pub fn end_if(&mut self) -> Result<(), String> {
        let idx = self.jumps.pop()
            .ok_or_else(|| "Pila de jumps vacía al cerrar si".to_string())?;
        let target = self.quads.len();
        self.fill_jump(idx, target);
        Ok(())
    }

    /// Marca el punto de retorno del while ANTES de evaluar la condición.
    pub fn push_return_point(&mut self) {
        self.jumps.push(self.quads.len());
    }

    /// Cierra el while: emite GOTO al punto de retorno y rellena el GOTOF.
    pub fn end_while(&mut self) -> Result<(), String> {
        let gotof_idx = self.jumps.pop()
            .ok_or_else(|| "Pila de jumps vacía al cerrar mientras".to_string())?;
        let return_idx = self.jumps.pop()
            .ok_or_else(|| "Punto de retorno faltante en mientras".to_string())?;
        self.quads.push(Quadruple::new(Op::Goto, "_", "_", return_idx.to_string()));
        let target = self.quads.len();
        self.fill_jump(gotof_idx, target);
        Ok(())
    }

    // ── Diagnóstico ───────────────────────────────────────────────────────────

    pub fn print(&self) {
        println!("  ┌──────────────────────────────────────────────────────┐");
        println!("  │  FILA DE CUÁDRUPLOS                                  │");
        println!("  └──────────────────────────────────────────────────────┘");
        if self.quads.is_empty() {
            println!("  (vacía)");
            return;
        }
        for (i, q) in self.quads.iter().enumerate() {
            println!("  {:>3}: {}", i, q);
        }
        println!();
    }

    /// Sanity check: al terminar el parse las pilas auxiliares deben estar vacías.
    pub fn is_clean(&self) -> bool {
        self.operands.is_empty() && self.operators.is_empty() && self.jumps.is_empty()
    }

    pub fn dump_stacks_if_dirty(&self) {
        if !self.is_clean() {
            eprintln!("  [WARN] pilas auxiliares no quedaron vacías:");
            eprintln!("    operands:  {:?}",
                self.operands.iter().map(|o| o.address).collect::<Vec<_>>());
            eprintln!("    operators: {:?}", self.operators);
            eprintln!("    jumps:     {:?}", self.jumps);
        }
    }
}
