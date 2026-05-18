// Análisis semántico de Patito: cubo, directorio de funciones,
// tablas de variables y estado del análisis.

use std::collections::HashMap;
use std::fmt;

// ─── Tipos ────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tipo {
    Entero,
    Flotante,
    Nula,
}

impl fmt::Display for Tipo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tipo::Entero   => write!(f, "entero"),
            Tipo::Flotante => write!(f, "flotante"),
            Tipo::Nula     => write!(f, "nula"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoOp {
    Mas, Menos, Por, Div,
    Menor, Mayor, Igual, Dif,
    Asig,
}

impl fmt::Display for TipoOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TipoOp::Mas => "+", TipoOp::Menos => "-",
            TipoOp::Por => "*", TipoOp::Div => "/",
            TipoOp::Menor => "<", TipoOp::Mayor => ">",
            TipoOp::Igual => "==", TipoOp::Dif => "!=",
            TipoOp::Asig => "=",
        };
        write!(f, "{}", s)
    }
}

// ─── Errores semánticos ───────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticError {
    VariableDuplicada(String),
    VariableNoDeclarada(String),
    FuncionDuplicada(String),
    FuncionNoDeclarada(String),
    TiposIncompatibles { izq: Tipo, der: Tipo, op: TipoOp },
    RetornoIncompatible { esperado: Tipo, obtenido: Tipo },
    RetornoEnFuncionNula,
    RegresaFueraDeFuncion,
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticError::VariableDuplicada(n) =>
                write!(f, "variable '{}' declarada dos veces en el mismo scope", n),
            SemanticError::VariableNoDeclarada(n) =>
                write!(f, "variable '{}' no declarada", n),
            SemanticError::FuncionDuplicada(n) =>
                write!(f, "función '{}' declarada dos veces", n),
            SemanticError::FuncionNoDeclarada(n) =>
                write!(f, "función '{}' no declarada", n),
            SemanticError::TiposIncompatibles { izq, der, op } =>
                write!(f, "tipos incompatibles: {} {} {}", izq, op, der),
            SemanticError::RetornoIncompatible { esperado, obtenido } =>
                write!(f, "regresa: se esperaba {}, se obtuvo {}", esperado, obtenido),
            SemanticError::RetornoEnFuncionNula =>
                write!(f, "regresa no permitido en función con tipo de retorno nula"),
            SemanticError::RegresaFueraDeFuncion =>
                write!(f, "regresa solo se permite dentro de funciones"),
        }
    }
}

impl std::error::Error for SemanticError {}

// ─── Cubo semántico ───────────────────────────────────────────────────
// Devuelve el tipo resultante de aplicar `op` entre `izq` y `der`,
// o None si la combinación no es válida.
pub fn cubo(izq: Tipo, der: Tipo, op: TipoOp) -> Option<Tipo> {
    use Tipo::*;
    use TipoOp::*;

    match op {
        // Aritméticos: + - * /
        Mas | Menos | Por | Div => match (izq, der) {
            (Entero,   Entero)   => Some(Entero),
            (Flotante, Flotante) => Some(Flotante),
            (Entero,   Flotante) => Some(Flotante),
            (Flotante, Entero)   => Some(Flotante),
            _ => None,
        },

        // Relacionales: < > == !=  → resultado entero (0/1, sin tipo bool dedicado)
        Menor | Mayor | Igual | Dif => match (izq, der) {
            (Entero,   Entero)   => Some(Entero),
            (Flotante, Flotante) => Some(Entero),
            (Entero,   Flotante) => Some(Entero),
            (Flotante, Entero)   => Some(Entero),
            _ => None,
        },

        // Asignación: el lado derecho debe ser asignable al izquierdo.
        // Permitimos entero→entero, flotante→flotante, y entero→flotante (promoción).
        // NO permitimos flotante→entero (pérdida de precisión).
        Asig => match (izq, der) {
            (Entero,   Entero)   => Some(Entero),
            (Flotante, Flotante) => Some(Flotante),
            (Flotante, Entero)   => Some(Flotante),
            _ => None,
        },
    }
}

// ─── VarInfo y VarTable ───────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub tipo: Tipo,
}

#[derive(Debug, Clone, Default)]
pub struct VarTable {
    vars: HashMap<String, VarInfo>,
}

impl VarTable {
    pub fn nueva() -> Self { VarTable::default() }

    pub fn declarar(&mut self, nombre: String, tipo: Tipo) -> Result<(), SemanticError> {
        if self.vars.contains_key(&nombre) {
            return Err(SemanticError::VariableDuplicada(nombre));
        }
        self.vars.insert(nombre, VarInfo { tipo });
        Ok(())
    }

    pub fn buscar(&self, nombre: &str) -> Option<&VarInfo> {
        self.vars.get(nombre)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize { self.vars.len() }

    #[allow(dead_code)]
    pub fn iter_vars(&self) -> impl Iterator<Item = (&String, &VarInfo)> {
        self.vars.iter()
    }
}

// ─── FuncInfo y DirFunc ───────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub tipo_retorno: Tipo,
    pub params: Vec<(String, Tipo)>,
    pub vars: VarTable,
}

#[derive(Debug, Clone, Default)]
pub struct DirFunc {
    funcs: HashMap<String, FuncInfo>,
}

impl DirFunc {
    pub fn nuevo() -> Self { DirFunc::default() }

    pub fn declarar(&mut self, nombre: String, tipo_retorno: Tipo) -> Result<(), SemanticError> {
        if self.funcs.contains_key(&nombre) {
            return Err(SemanticError::FuncionDuplicada(nombre));
        }
        self.funcs.insert(nombre, FuncInfo {
            tipo_retorno,
            params: Vec::new(),
            vars: VarTable::nueva(),
        });
        Ok(())
    }

    pub fn buscar(&self, nombre: &str) -> Option<&FuncInfo> {
        self.funcs.get(nombre)
    }

    pub fn buscar_mut(&mut self, nombre: &str) -> Option<&mut FuncInfo> {
        self.funcs.get_mut(nombre)
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = (&String, &FuncInfo)> {
        self.funcs.iter()
    }
}

// ─── SemanticState ────────────────────────────────────────────────────
pub struct SemanticState {
    pub dir_func: DirFunc,
    pub scope_actual: String,
}

impl SemanticState {
    pub fn nuevo() -> Self {
        SemanticState {
            dir_func: DirFunc::nuevo(),
            scope_actual: String::new(),
        }
    }

    // PN1 — Iniciar programa
    pub fn iniciar_programa(&mut self, _nombre: &str) -> Result<(), SemanticError> {
        self.dir_func.declarar("global".to_string(), Tipo::Nula)?;
        self.scope_actual = "global".to_string();
        Ok(())
    }

    // PN2 — Declarar variable
    pub fn declarar_variable(&mut self, nombre: String, tipo: Tipo)
        -> Result<(), SemanticError>
    {
        let func = self.dir_func.buscar_mut(&self.scope_actual)
            .expect("scope actual debe existir");
        func.vars.declarar(nombre, tipo)
    }

    // PN3 — Declarar función (entrada)
    pub fn declarar_funcion(&mut self, nombre: String, tipo_retorno: Tipo)
        -> Result<(), SemanticError>
    {
        self.dir_func.declarar(nombre.clone(), tipo_retorno)?;
        self.scope_actual = nombre;
        Ok(())
    }

    // PN4 — Declarar parámetro
    pub fn declarar_parametro(&mut self, nombre: String, tipo: Tipo)
        -> Result<(), SemanticError>
    {
        let func = self.dir_func.buscar_mut(&self.scope_actual)
            .expect("scope actual debe existir");
        func.params.push((nombre.clone(), tipo));
        func.vars.declarar(nombre, tipo)
    }

    // PN5 — Cerrar función (salida)
    pub fn cerrar_funcion(&mut self) {
        self.scope_actual = "global".to_string();
    }

    // PN6 — Buscar variable (cascada local → global)
    pub fn buscar_variable(&self, nombre: &str) -> Result<Tipo, SemanticError> {
        if let Some(func) = self.dir_func.buscar(&self.scope_actual) {
            if let Some(v) = func.vars.buscar(nombre) {
                return Ok(v.tipo);
            }
        }
        if self.scope_actual != "global" {
            if let Some(global) = self.dir_func.buscar("global") {
                if let Some(v) = global.vars.buscar(nombre) {
                    return Ok(v.tipo);
                }
            }
        }
        Err(SemanticError::VariableNoDeclarada(nombre.to_string()))
    }

    // PN7 — Buscar función
    pub fn buscar_funcion(&self, nombre: &str) -> Result<Tipo, SemanticError> {
        self.dir_func.buscar(nombre)
            .map(|f| f.tipo_retorno)
            .ok_or_else(|| SemanticError::FuncionNoDeclarada(nombre.to_string()))
    }

    // PN8, PN9, PN10 — Validar operación binaria (suma/resta, mult/div, relacional)
    pub fn validar_operacion(&self, izq: Tipo, der: Tipo, op: TipoOp)
        -> Result<Tipo, SemanticError>
    {
        cubo(izq, der, op)
            .ok_or(SemanticError::TiposIncompatibles { izq, der, op })
    }

    // PN11 — Validar asignación
    pub fn validar_asignacion(&self, var: &str, tipo_expr: Tipo)
        -> Result<(), SemanticError>
    {
        let tipo_var = self.buscar_variable(var)?;
        cubo(tipo_var, tipo_expr, TipoOp::Asig)
            .map(|_| ())
            .ok_or(SemanticError::TiposIncompatibles {
                izq: tipo_var, der: tipo_expr, op: TipoOp::Asig
            })
    }

    // PN12 — Validar retorno
    pub fn validar_regresa(&self, tipo_expr: Tipo) -> Result<(), SemanticError> {
        if self.scope_actual == "global" || self.scope_actual.is_empty() {
            return Err(SemanticError::RegresaFueraDeFuncion);
        }
        let func = self.dir_func.buscar(&self.scope_actual)
            .expect("scope actual debe existir");
        if func.tipo_retorno == Tipo::Nula {
            return Err(SemanticError::RetornoEnFuncionNula);
        }
        if func.tipo_retorno != tipo_expr {
            return Err(SemanticError::RetornoIncompatible {
                esperado: func.tipo_retorno,
                obtenido: tipo_expr,
            });
        }
        Ok(())
    }
}

// ─── Tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubo_aritmetica_basica() {
        assert_eq!(cubo(Tipo::Entero, Tipo::Entero, TipoOp::Mas), Some(Tipo::Entero));
        assert_eq!(cubo(Tipo::Entero, Tipo::Flotante, TipoOp::Por), Some(Tipo::Flotante));
        assert_eq!(cubo(Tipo::Nula, Tipo::Entero, TipoOp::Mas), None);
    }

    #[test]
    fn cubo_asignacion() {
        assert_eq!(cubo(Tipo::Flotante, Tipo::Entero, TipoOp::Asig), Some(Tipo::Flotante));
        assert_eq!(cubo(Tipo::Entero, Tipo::Flotante, TipoOp::Asig), None);
    }

    #[test]
    fn variable_duplicada() {
        let mut t = VarTable::nueva();
        assert!(t.declarar("x".into(), Tipo::Entero).is_ok());
        assert!(matches!(
            t.declarar("x".into(), Tipo::Entero),
            Err(SemanticError::VariableDuplicada(_))
        ));
    }

    #[test]
    fn cascada_local_global() {
        let mut sem = SemanticState::nuevo();
        sem.iniciar_programa("test").unwrap();
        sem.declarar_variable("g".into(), Tipo::Entero).unwrap();
        sem.declarar_funcion("foo".into(), Tipo::Entero).unwrap();
        sem.declarar_variable("local".into(), Tipo::Flotante).unwrap();

        // Desde foo, ve la local y la global
        assert_eq!(sem.buscar_variable("local").unwrap(), Tipo::Flotante);
        assert_eq!(sem.buscar_variable("g").unwrap(), Tipo::Entero);

        // De vuelta a global, no ve la local de foo
        sem.cerrar_funcion();
        assert!(sem.buscar_variable("local").is_err());
        assert_eq!(sem.buscar_variable("g").unwrap(), Tipo::Entero);
    }
}