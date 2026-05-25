use std::collections::HashMap;
use crate::types::Type;

// ── Entrada de variable ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub var_type: Type,
}

// ── Entrada de función ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub return_type: Type,
    /// Parámetros en orden de declaración.
    pub params: Vec<(String, Type)>,
    /// Variables locales + parámetros (acceso en O(1) por nombre).
    pub local_vars: HashMap<String, VarInfo>,
}

impl FuncInfo {
    pub fn new(return_type: Type, params: Vec<(String, Type)>) -> Self {
        FuncInfo { return_type, params, local_vars: HashMap::new() }
    }
}

// ── Directorio de funciones ───────────────────────────────────────────────────
//
// Estructura principal del análisis semántico de Patito.
//
// Organización:
//   · global_vars : HashMap<String, VarInfo>
//       Variables declaradas en el bloque `vars` del programa principal.
//       Acceso O(1) por nombre; la inserción detecta duplicados en el llamador.
//
//   · funcs : HashMap<String, FuncInfo>
//       Una entrada por cada función declarada.
//       Cada FuncInfo lleva su propio HashMap de variables locales (incluyendo
//       parámetros) para que la búsqueda por nombre también sea O(1).
//
// Por qué HashMap y no árbol/lista:
//   · Las operaciones críticas son lookup e inserción, ambas O(1) con hash.
//   · El orden de declaración no importa para la semántica; solo el nombre.
//   · Los parámetros se guardan además en un Vec<(String,Type)> para preservar
//     el orden (necesario al verificar llamadas en entregas posteriores).

#[derive(Debug)]
pub struct FuncDir {
    pub global_vars: HashMap<String, VarInfo>,
    pub funcs: HashMap<String, FuncInfo>,
}

impl Default for FuncDir {
    fn default() -> Self {
        FuncDir { global_vars: HashMap::new(), funcs: HashMap::new() }
    }
}

impl FuncDir {
    pub fn new() -> Self { Self::default() }

    pub fn print(&self) {
        println!("  ┌──────────────────────────────────────────────────────┐");
        println!("  │  DIRECTORIO DE FUNCIONES                             │");
        println!("  └──────────────────────────────────────────────────────┘");

        // ── Variables globales
        println!("\n  [scope global]");
        println!("  Variables globales:");
        if self.global_vars.is_empty() {
            println!("    (ninguna)");
        } else {
            let mut gv: Vec<_> = self.global_vars.iter().collect();
            gv.sort_by_key(|(k, _)| k.as_str());
            for (name, info) in gv {
                println!("    {:<16} : {}", name, info.var_type);
            }
        }

        // ── Funciones
        let mut funcs: Vec<_> = self.funcs.iter().collect();
        funcs.sort_by_key(|(k, _)| k.as_str());
        for (fname, info) in funcs {
            let params_str = info.params.iter()
                .map(|(n, t)| format!("{}: {}", n, t))
                .collect::<Vec<_>>()
                .join(", ");
            println!("\n  [función '{}']", fname);
            println!("  Retorna   : {}", info.return_type);
            println!("  Parámetros: ({})", params_str);

            // Separar parámetros de variables locales declaradas con `vars`
            let param_names: std::collections::HashSet<&str> =
                info.params.iter().map(|(n, _)| n.as_str()).collect();

            println!("  Vars locales (excluye parámetros):");
            let mut locals: Vec<_> = info.local_vars.iter()
                .filter(|(k, _)| !param_names.contains(k.as_str()))
                .collect();

            if locals.is_empty() {
                println!("    (ninguna)");
            } else {
                locals.sort_by_key(|(k, _)| k.as_str());
                for (n, v) in locals {
                    println!("    {:<16} : {}", n, v.var_type);
                }
            }
        }
        println!();
    }
}
