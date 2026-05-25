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
//   1. Resuelve datos del scope (tipo de una variable, return_type de la función)
//   2. Delega la mecánica al QuadGen interno
//   3. Captura los Result::Err que devuelve QuadGen y los empuja a errors

pub struct SemanticContext {
    pub func_dir:     FuncDir,
    pub errors:       Vec<SemanticError>,
    current_func:     Option<String>,
    pub qg:           QuadGen,
}

impl SemanticContext {
    pub fn new() -> Self {
        SemanticContext {
            func_dir:     FuncDir::new(),
            errors:       Vec::new(),
            current_func: None,
            qg:           QuadGen::new(),
        }
    }

    pub fn has_errors(&self) -> bool { !self.errors.is_empty() }

    // ── Helpers de scope ──────────────────────────────────────────────────────

    /// Busca una variable: primero en el scope local activo, luego en globales.
    /// Type es Copy, así que retornar por valor evita conflictos de borrow
    /// cuando luego mutamos self.qg.
    fn lookup_var(&self, name: &str) -> Option<Type> {
        if let Some(func_name) = self.current_func.as_deref() {
            if let Some(func) = self.func_dir.funcs.get(func_name) {
                if let Some(v) = func.local_vars.get(name) {
                    return Some(v.var_type);
                }
            }
        }
        self.func_dir.global_vars.get(name).map(|v| v.var_type)
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
    // ═════════════════════════════════════════════════════════════════════════

    /// PN1 — inicio de programa.
    pub fn start_program(&mut self, _name: String) {
        // Reservado para futuras entregas.
    }

    /// PN2 — cabecera de función. Registra la función e instala scope local.
    pub fn begin_func(&mut self, name: String, return_type: Type, params: Vec<(String, Type)>) {
        if self.func_dir.funcs.contains_key(&name) {
            self.push_err(format!("Función '{}' declarada más de una vez", name));
        }

        let mut info = FuncInfo::new(return_type, params.clone());
        for (pname, ptype) in &params {
            if info.local_vars.contains_key(pname) {
                self.push_err(format!("Parámetro '{}' duplicado en función '{}'", pname, name));
            } else {
                info.local_vars.insert(pname.clone(), VarInfo { var_type: *ptype });
            }
        }

        self.func_dir.funcs.insert(name.clone(), info);
        self.current_func = Some(name);
    }

    /// PN3 — fin de función. Restaura scope global.
    pub fn end_func(&mut self) {
        self.current_func = None;
    }

    /// PN4 — declaración de variable.
    pub fn declare_var(&mut self, name: String, var_type: Type) {
        match self.current_func.as_deref() {
            None => {
                if self.func_dir.global_vars.contains_key(&name) {
                    self.push_err(format!("Variable global '{}' declarada más de una vez", name));
                } else {
                    self.func_dir.global_vars.insert(name, VarInfo { var_type });
                }
            }
            Some(func_name) => {
                let func_name = func_name.to_owned();
                if let Some(func) = self.func_dir.funcs.get_mut(&func_name) {
                    if func.local_vars.contains_key(&name) {
                        self.push_err(format!(
                            "Variable '{}' declarada más de una vez en función '{}'",
                            name, func_name
                        ));
                    } else {
                        func.local_vars.insert(name, VarInfo { var_type });
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
    // ═════════════════════════════════════════════════════════════════════════

    // ── Operandos ─────────────────────────────────────────────────────────────

    /// PN: id usado como operando (en Factor o como LHS de asignación).
    /// Resuelve el tipo consultando func_dir antes de empujar.
    pub fn push_id_operand(&mut self, name: String) {
        match self.lookup_var(&name) {
            Some(ty) => self.qg.push_operand(Operand { name, ty }),
            None => self.push_err(format!("Variable '{}' no declarada", name)),
        }
    }

    /// PN: constante entera literal.
    pub fn push_const_int(&mut self, v: i64) {
        self.qg.push_operand(Operand { name: v.to_string(), ty: Type::Entero });
    }

    /// PN: constante flotante literal.
    pub fn push_const_float(&mut self, v: f64) {
        self.qg.push_operand(Operand { name: v.to_string(), ty: Type::Flotante });
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
    /// return_type de la función actual.
    pub fn emit_return(&mut self) {
        let expected = self.current_return_type();
        match self.qg.emit_return() {
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
}
