use crate::types::Type;
use crate::func_dir::{FuncDir, FuncInfo, VarInfo};
use crate::quad_gen::{QuadGen, Operand};
use crate::semantic_cube::{Op, is_assignable};

// ── Error semántico ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SemanticError(pub String);

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error semántico: {}", self.0)
    }
}

// ── Contexto semántico ────────────────────────────────────────────────────────
//
// Se pasa como parámetro mutable al parser (grammar.lalrpop).
// Coordina las dos partes del análisis:
//
//   · Tablas y scope    — func_dir, current_func (Entrega 2)
//   · Generación de IR  — qg: QuadGen           (Entrega 3)
//
// SemanticContext es el ÚNICO objeto al que llama la gramática. Cuando un
// punto neurálgico necesita generar cuádruplos, este wrapper:
//   1. Resuelve datos del scope (tipo y dirección de una variable, return_type)
//   2. Delega la mecánica al QuadGen interno
//   3. Captura los Result::Err que devuelve QuadGen y los empuja a errors
//
// Entrega 4: lookup_var ahora resuelve también la dirección de memoria virtual,
// y las declaraciones de variables / parámetros invocan a func_dir para alocar
// direcciones. push_const_int/float pasan por la ConstTable para deduplicar.

// ── Contexto de una llamada a función ─────────────────────────────────────────
//
// Entrega 4 Parte 2: cada vez que se ve `id (` se hace push de un CallCtx en
// la pila de llamadas, y al cerrar `)` se hace pop. La pila permite llamadas
// anidadas como `foo(bar(3), 5)`: cuando empieza `bar`, su contador `k` no
// pisa al de `foo`. El nombre se guarda para validar args y emitir GOSUB.

#[derive(Debug)]
struct CallCtx {
    name: String,
    k:    u32,      // índice 1-based del próximo parámetro a procesar
}

pub struct SemanticContext {
    pub func_dir:     FuncDir,
    pub errors:       Vec<SemanticError>,
    current_func:     Option<String>,
    pub qg:           QuadGen,
    call_stack:       Vec<CallCtx>,
}

impl SemanticContext {
    pub fn new() -> Self {
        SemanticContext {
            func_dir:     FuncDir::new(),
            errors:       Vec::new(),
            current_func: None,
            qg:           QuadGen::new(),
            call_stack:   Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool { !self.errors.is_empty() }

    // ── Helpers de scope ──────────────────────────────────────────────────────

    /// Busca una variable: primero en el scope local activo, luego en globales.
    /// Devuelve (tipo, dirección) para que el llamador pueda construir un
    /// Operand listo para empujar a la pila de operandos.
    /// Type es Copy y u32 también, así que retornar por valor evita conflictos
    /// de borrow cuando luego mutamos self.qg.
    fn lookup_var(&self, name: &str) -> Option<(Type, u32)> {
        if let Some(func_name) = self.current_func.as_deref() {
            if let Some(func) = self.func_dir.funcs.get(func_name) {
                if let Some(v) = func.local_vars.get(name) {
                    return Some((v.var_type, v.address));
                }
            }
        }
        self.func_dir.global_vars.get(name).map(|v| (v.var_type, v.address))
    }

    fn current_return_type(&self) -> Type {
        self.current_func.as_deref()
            .and_then(|f| self.func_dir.funcs.get(f))
            .map(|f| f.return_type)
            .unwrap_or(Type::Nula)
    }

    fn push_err(&mut self, msg: impl Into<String>) {
        self.errors.push(SemanticError(msg.into()));
    }

    // ═════════════════════════════════════════════════════════════════════════
    //  PUNTOS NEURÁLGICOS — entrega 2 (scope y declaraciones)
    //  Entrega 4: declare_var y begin_func ahora alocan direcciones de memoria.
    // ═════════════════════════════════════════════════════════════════════════

    /// PN1 — inicio de programa. Emite el GOTO inicial con destino pendiente
    /// para que la ejecución brinque sobre los cuerpos de las funciones y
    /// caiga en el bloque 'inicio'. El back-patch se hace en patch_main_goto.
    pub fn start_program(&mut self, _name: String) {
        self.qg.emit_initial_goto();
    }

    /// PN — rellena el GOTO inicial con la posición actual (primer cuádruplo
    /// del bloque 'inicio'). Se invoca desde el marker MainStart de la gramática.
    pub fn patch_main_goto(&mut self) {
        if let Err(e) = self.qg.patch_main_goto() {
            self.push_err(e);
        }
    }

    /// PN2 — cabecera de función. Registra la función, instala scope local,
    /// resetea contadores de temporales, asigna start_quad y return_addr (si
    /// no es nula) y asigna direcciones a los parámetros.
    pub fn begin_func(&mut self, name: String, return_type: Type, params: Vec<(String, Type)>) {
        if self.func_dir.funcs.contains_key(&name) {
            self.push_err(format!("Función '{}' declarada más de una vez", name));
        }

        // Reservar la dirección global donde la función escribirá su retorno
        // (None si la función es nula — no se reserva nada).
        let return_addr = match return_type {
            Type::Nula => None,
            _          => Some(self.func_dir.alloc_global(return_type)),
        };

        // Registrar la función con FuncInfo vacío. Los parámetros se agregan abajo
        // vía alloc_local, que necesita que la entrada ya exista en el directorio
        // para poder incrementar los contadores n_local_*.
        let mut info = FuncInfo::new(return_type, params.clone());
        info.start_quad  = self.qg.quads.len();
        info.return_addr = return_addr;
        self.func_dir.funcs.insert(name.clone(), info);
        self.current_func = Some(name.clone());

        // Reset de temporales: cada función empieza desde TEMP_*_BASE.
        self.qg.reset_temp_counters();

        // Asignar dirección local a cada parámetro (preservando el orden).
        for (pname, ptype) in &params {
            let dup = self.func_dir.funcs.get(&name)
                .map(|f| f.local_vars.contains_key(pname)).unwrap_or(false);
            if dup {
                self.push_err(format!(
                    "Parámetro '{}' duplicado en función '{}'", pname, name
                ));
            } else {
                let addr = self.func_dir.alloc_local(&name, *ptype);
                if let Some(func) = self.func_dir.funcs.get_mut(&name) {
                    func.local_vars.insert(
                        pname.clone(),
                        VarInfo { var_type: *ptype, address: addr },
                    );
                }
            }
        }
    }

    /// PN3 — fin de función. Guarda el snapshot de temps en el FuncInfo y
    /// restaura scope global.
    pub fn end_func(&mut self) {
        if let Some(name) = self.current_func.take() {
            let (ti, tf) = self.qg.take_temp_counts();
            if let Some(func) = self.func_dir.funcs.get_mut(&name) {
                func.n_temp_int   = ti;
                func.n_temp_float = tf;
            }
        }
    }

    /// PN4 — declaración de variable. Asigna dirección global o local según el
    /// scope actual.
    pub fn declare_var(&mut self, name: String, var_type: Type) {
        match self.current_func.as_deref() {
            None => {
                if self.func_dir.global_vars.contains_key(&name) {
                    self.push_err(format!(
                        "Variable global '{}' declarada más de una vez", name
                    ));
                } else {
                    let addr = self.func_dir.alloc_global(var_type);
                    self.func_dir.global_vars.insert(
                        name,
                        VarInfo { var_type, address: addr },
                    );
                }
            }
            Some(func_name) => {
                let func_name = func_name.to_owned();
                let dup = self.func_dir.funcs.get(&func_name)
                    .map(|f| f.local_vars.contains_key(&name)).unwrap_or(false);
                if dup {
                    self.push_err(format!(
                        "Variable '{}' declarada más de una vez en función '{}'",
                        name, func_name
                    ));
                } else {
                    let addr = self.func_dir.alloc_local(&func_name, var_type);
                    if let Some(func) = self.func_dir.funcs.get_mut(&func_name) {
                        func.local_vars.insert(
                            name,
                            VarInfo { var_type, address: addr },
                        );
                    }
                }
            }
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    //  PUNTOS NEURÁLGICOS — entrega 3 (cuádruplos)
    //
    //  Patrón común:  el wrapper hace lookups/validaciones de scope,
    //  delega la mecánica al qg, y atrapa cualquier Err.
    //
    //  Entrega 4: los operandos ahora llevan dirección en lugar de nombre.
    // ═════════════════════════════════════════════════════════════════════════

    // ── Operandos ─────────────────────────────────────────────────────────────

    /// PN: id usado como operando (en Factor o como LHS de asignación).
    /// Resuelve (tipo, dirección) consultando func_dir antes de empujar.
    pub fn push_id_operand(&mut self, name: String) {
        match self.lookup_var(&name) {
            Some((ty, address)) => self.qg.push_operand(Operand { address, ty }),
            None => self.push_err(format!("Variable '{}' no declarada", name)),
        }
    }

    /// PN: constante entera literal. Pasa por ConstTable para deduplicar.
    pub fn push_const_int(&mut self, v: i64) {
        let address = self.func_dir.intern_int(v);
        self.qg.push_operand(Operand { address, ty: Type::Entero });
    }

    /// PN: constante flotante literal. Pasa por ConstTable para deduplicar.
    pub fn push_const_float(&mut self, v: f64) {
        let address = self.func_dir.intern_float(v);
        self.qg.push_operand(Operand { address, ty: Type::Flotante });
    }

    // ── Operadores ────────────────────────────────────────────────────────────

    /// PN: operador binario o asignación. Puede provocar colapsos por precedencia.
    pub fn push_operator(&mut self, op: Op) {
        if let Err(e) = self.qg.push_operator(op) {
            self.push_err(e);
        }
    }

    // ── Paréntesis ────────────────────────────────────────────────────────────

    pub fn push_lparen(&mut self) {
        self.qg.push_lparen();
    }

    pub fn pop_lparen(&mut self) {
        if let Err(e) = self.qg.pop_lparen() {
            self.push_err(e);
        }
    }

    // ── Statements lineales ───────────────────────────────────────────────────

    /// PN: cierre de asignación (al ver el ';'). Fuerza colapso de todo
    /// hasta dejar consumido el '=' que estaba en el fondo de la pila.
    pub fn end_assignment(&mut self) {
        if let Err(e) = self.qg.end_assignment() {
            self.push_err(e);
        }
    }

    /// PN: escribe(expr) — un cuádruplo PRINT por cada elemento.
    pub fn emit_print_expr(&mut self) {
        if let Err(e) = self.qg.emit_print_expr() {
            self.push_err(e);
        }
    }

    /// PN: escribe("literal") — cuádruplo PRINT_STR directo, sin tocar pila.
    pub fn emit_print_str(&mut self, s: String) {
        self.qg.emit_print_str(s);
    }

    /// PN: regresa(expr). Valida que el tipo del valor sea asignable al
    /// return_type de la función actual y emite RETURN escribiendo a return_addr.
    pub fn emit_return(&mut self) {
        let expected = self.current_return_type();
        let return_addr = self.current_func.as_deref()
            .and_then(|f| self.func_dir.funcs.get(f))
            .and_then(|f| f.return_addr);
        match self.qg.emit_return(return_addr) {
            Err(e) => self.push_err(e),
            Ok(operand) => {
                if expected == Type::Nula {
                    self.push_err(format!(
                        "regresa con valor en función de tipo nula"
                    ));
                } else if !is_assignable(expected, operand.ty) {
                    self.push_err(format!(
                        "regresa retorna {} pero la función espera {}",
                        operand.ty, expected
                    ));
                }
            }
        }
    }

    // ── Control de flujo ──────────────────────────────────────────────────────

    /// PN tras evaluar la condición del 'si': fuerza colapso, valida tipo,
    /// emite GOTOF pendiente y empuja su índice a jumps.
    pub fn gen_gotof_if(&mut self) {
        match self.qg.gen_gotof() {
            Err(e) => self.push_err(e),
            Ok(ty) if ty != Type::Entero => {
                self.push_err(format!(
                    "La condición del 'si' debe ser entero, se obtuvo {}", ty
                ));
            }
            Ok(_) => {}
        }
    }

    /// PN al ver 'sino': emite GOTO pendiente, rellena el GOTOF previo.
    pub fn gen_else_goto(&mut self) {
        if let Err(e) = self.qg.gen_else_goto() {
            self.push_err(e);
        }
    }

    /// PN al cerrar el 'si' (con o sin sino): rellena salto pendiente.
    pub fn end_if(&mut self) {
        if let Err(e) = self.qg.end_if() {
            self.push_err(e);
        }
    }

    /// PN antes de evaluar la cond del 'mientras': marca punto de retorno.
    pub fn push_return_point(&mut self) {
        self.qg.push_return_point();
    }

    /// PN tras evaluar la cond del 'mientras': igual que gen_gotof_if.
    pub fn gen_gotof_while(&mut self) {
        match self.qg.gen_gotof() {
            Err(e) => self.push_err(e),
            Ok(ty) if ty != Type::Entero => {
                self.push_err(format!(
                    "La condición del 'mientras' debe ser entero, se obtuvo {}", ty
                ));
            }
            Ok(_) => {}
        }
    }

    /// PN al cerrar el 'mientras': emite GOTO al inicio y rellena GOTOF.
    pub fn end_while(&mut self) {
        if let Err(e) = self.qg.end_while() {
            self.push_err(e);
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    //  PUNTOS NEURÁLGICOS — entrega 4 parte 2 (cuádruplos de funciones)
    // ═════════════════════════════════════════════════════════════════════════

    /// PN al cerrar el cuerpo de la función: emite ENDFUNC y restaura scope.
    /// La emisión va ANTES de end_func() para que el ENDFUNC vea el snapshot
    /// de temporales correcto.
    pub fn gen_endfunc(&mut self) {
        self.qg.emit_endfunc();
    }

    /// PN al ver el nombre de la función en una llamada (CallStmtHead /
    /// FactorCallHead). Valida que la función existe, emite ERA, empuja un
    /// LParen como fondo falso a la pila de operadores (para que el
    /// force_collapse de cada PARAM no toque operadores de la expresión que
    /// envuelve a la llamada), y empuja un nuevo CallCtx con k = 1.
    pub fn gen_era(&mut self, name: String) {
        if !self.func_dir.funcs.contains_key(&name) {
            self.push_err(format!("Función '{}' no declarada", name));
            // Aun así empujamos un CallCtx (y el LParen) para que el resto de
            // la llamada no explote con "PARAM fuera de llamada". Los PARAMs
            // y el GOSUB también van a fallar, pero el error ya quedó registrado.
            self.qg.push_lparen();
            self.call_stack.push(CallCtx { name, k: 1 });
            return;
        }
        self.qg.emit_era(&name);
        self.qg.push_lparen();
        self.call_stack.push(CallCtx { name, k: 1 });
    }

    /// PN tras evaluar cada argumento de una llamada. Colapsa la expresión,
    /// hace pop del operando resultante, valida su tipo contra el k-ésimo
    /// parámetro de la función, emite PARAM con la dirección del parámetro
    /// destino, y avanza k.
    pub fn gen_param(&mut self) {
        if let Err(e) = self.qg.force_collapse() {
            self.push_err(e);
            return;
        }
        let arg = match self.qg.operands.pop() {
            Some(o) => o,
            None    => { self.push_err("Falta argumento en llamada"); return; }
        };

        // Snapshot inmutable de la información que necesitamos antes de tocar
        // la pila de llamadas (que tiene borrow mutable).
        let (name, k) = match self.call_stack.last() {
            Some(c) => (c.name.clone(), c.k),
            None    => { self.push_err("PARAM fuera de llamada"); return; }
        };

        let func = match self.func_dir.funcs.get(&name) {
            Some(f) => f,
            None    => { return; /* error ya registrado en gen_era */ }
        };

        if (k as usize) > func.params.len() {
            self.push_err(format!(
                "Demasiados argumentos para '{}': esperaba {}", name, func.params.len()
            ));
            return;
        }

        let (pname, param_ty) = func.params[(k - 1) as usize].clone();
        let param_addr = match func.local_vars.get(&pname) {
            Some(v) => v.address,
            None    => { self.push_err("Parámetro sin dirección"); return; }
        };

        if !is_assignable(param_ty, arg.ty) {
            self.push_err(format!(
                "Argumento {} de '{}': se esperaba {}, se obtuvo {}",
                k, name, param_ty, arg.ty
            ));
        }

        self.qg.emit_param(arg.address, param_addr);
        if let Some(top) = self.call_stack.last_mut() {
            top.k += 1;
        }
    }

    /// PN al cerrar ')' en CallStmt: hace pop del LParen-fondo-falso, valida
    /// el número de args, emite GOSUB, y hace pop del CallCtx. No empuja
    /// operando (es sentencia).
    pub fn gen_gosub_stmt(&mut self) {
        // Pop del LParen empujado por gen_era. Cualquier operador sobrante
        // dentro de la llamada (no debería haber ninguno gracias a los
        // gen_param previos) se colapsaría aquí.
        if let Err(e) = self.qg.pop_lparen() {
            self.push_err(e);
        }
        let ctx = match self.call_stack.pop() {
            Some(c) => c,
            None    => { self.push_err("GOSUB fuera de llamada"); return; }
        };
        let (start, expected) = match self.func_dir.funcs.get(&ctx.name) {
            Some(f) => (f.start_quad, f.params.len() as u32),
            None    => return, // error ya registrado en gen_era
        };
        let got = ctx.k - 1;
        if got != expected {
            self.push_err(format!(
                "'{}' espera {} argumentos, recibió {}", ctx.name, expected, got
            ));
        }
        self.qg.emit_gosub(&ctx.name, start);
    }

    /// PN al cerrar ')' en FactorAtomCall: igual que gen_gosub_stmt + copia el
    /// valor de retorno a un temporal y lo empuja a la pila de operandos para
    /// que la expresión externa lo use.
    pub fn gen_gosub_factor(&mut self) {
        // Pop del LParen empujado por gen_era (igual que gen_gosub_stmt).
        if let Err(e) = self.qg.pop_lparen() {
            self.push_err(e);
        }
        let ctx = match self.call_stack.pop() {
            Some(c) => c,
            None    => { self.push_err("GOSUB fuera de llamada"); return; }
        };

        let (start, return_addr, return_type, expected) = match self.func_dir.funcs.get(&ctx.name) {
            Some(f) => (f.start_quad, f.return_addr, f.return_type, f.params.len() as u32),
            None    => return, // error ya registrado en gen_era
        };

        // Validación de número de args (idéntica a gen_gosub_stmt).
        let got = ctx.k - 1;
        if got != expected {
            self.push_err(format!(
                "'{}' espera {} argumentos, recibió {}", ctx.name, expected, got
            ));
        }

        // Si la función es nula no produce valor, no se puede usar como factor.
        let (addr, ty) = match (return_addr, return_type) {
            (Some(a), t) if t != Type::Nula => (a, t),
            _ => {
                self.push_err(format!(
                    "Función nula '{}' no puede usarse en una expresión", ctx.name
                ));
                // Aun así emitimos el GOSUB para mantener el contador de
                // cuádruplos consistente; el error queda registrado.
                self.qg.emit_gosub(&ctx.name, start);
                return;
            }
        };

        self.qg.emit_gosub(&ctx.name, start);
        // Copia return_addr → temporal nuevo, push temporal.
        let _ = self.qg.emit_return_value_copy(addr, ty);
    }
}
