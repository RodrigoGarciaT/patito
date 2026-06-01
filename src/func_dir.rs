use std::collections::HashMap;
use crate::types::{
    Type,
    GLOBAL_INT_BASE, GLOBAL_FLOAT_BASE,
    LOCAL_INT_BASE,  LOCAL_FLOAT_BASE,
    CTE_INT_BASE,    CTE_FLOAT_BASE,
};

// ── Entrada de variable ───────────────────────────────────────────────────────
//
// Entrega 4: además del tipo, cada variable conoce su dirección de memoria
// virtual, asignada al momento de declararla.

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub var_type: Type,
    pub address:  u32,
}

// ── Entrada de función ────────────────────────────────────────────────────────
//
// Entrega 4: además del tipo de retorno, la lista ordenada de parámetros y la
// tabla local de variables, cada función guarda:
//   · start_quad     — índice del primer cuádruplo del cuerpo (futuro GOSUB).
//   · n_local_*      — contadores de variables locales por tipo (para ERA).
//   · n_temp_*       — contadores de temporales por tipo (snapshot al cerrar
//                      la función desde QuadGen).

#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub return_type:   Type,
    /// Parámetros en orden de declaración.
    pub params:        Vec<(String, Type)>,
    /// Variables locales + parámetros (acceso en O(1) por nombre).
    pub local_vars:    HashMap<String, VarInfo>,
    pub start_quad:    usize,
    pub n_local_int:   u32,
    pub n_local_float: u32,
    pub n_temp_int:    u32,
    pub n_temp_float:  u32,
}

impl FuncInfo {
    pub fn new(return_type: Type, params: Vec<(String, Type)>) -> Self {
        FuncInfo {
            return_type,
            params,
            local_vars:    HashMap::new(),
            start_quad:    0,
            n_local_int:   0,
            n_local_float: 0,
            n_temp_int:    0,
            n_temp_float:  0,
        }
    }
}

// ── Tabla de constantes ───────────────────────────────────────────────────────
//
// Entrega 4: deduplica las constantes literales del programa entero. Si "5"
// aparece 10 veces, todas las ocurrencias comparten una sola dirección.
//
// f64 no implementa Hash, así que para los flotantes usamos la representación
// como String. Esto colapsa 3.1 con 3.10 (que es lo correcto semánticamente).

#[derive(Debug, Default)]
pub struct ConstTable {
    pub int_consts:   HashMap<i64, u32>,
    pub float_consts: HashMap<String, u32>,
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
//
// Entrega 4: además del directorio en sí, el FuncDir guarda la tabla de
// constantes y los contadores globales (que no son per-función, viven aquí).

#[derive(Debug)]
pub struct FuncDir {
    pub global_vars:       HashMap<String, VarInfo>,
    pub funcs:             HashMap<String, FuncInfo>,
    pub const_table:       ConstTable,
    pub next_global_int:   u32,
    pub next_global_float: u32,
    pub next_cte_int:      u32,
    pub next_cte_float:    u32,
}

impl Default for FuncDir {
    fn default() -> Self {
        FuncDir {
            global_vars:       HashMap::new(),
            funcs:             HashMap::new(),
            const_table:       ConstTable::default(),
            next_global_int:   GLOBAL_INT_BASE,
            next_global_float: GLOBAL_FLOAT_BASE,
            next_cte_int:      CTE_INT_BASE,
            next_cte_float:    CTE_FLOAT_BASE,
        }
    }
}

impl FuncDir {
    pub fn new() -> Self { Self::default() }

    // ── Asignación de direcciones ─────────────────────────────────────────────

    /// Reserva la siguiente dirección de variable global y la devuelve.
    pub fn alloc_global(&mut self, ty: Type) -> u32 {
        match ty {
            Type::Entero => {
                let a = self.next_global_int;
                self.next_global_int += 1;
                a
            }
            Type::Flotante => {
                let a = self.next_global_float;
                self.next_global_float += 1;
                a
            }
            Type::Nula => unreachable!("variable de tipo nula"),
        }
    }

    /// Reserva la siguiente dirección local dentro de una función.
    pub fn alloc_local(&mut self, func_name: &str, ty: Type) -> u32 {
        let f = self.funcs.get_mut(func_name).expect("función inexistente");
        match ty {
            Type::Entero => {
                let a = LOCAL_INT_BASE + f.n_local_int;
                f.n_local_int += 1;
                a
            }
            Type::Flotante => {
                let a = LOCAL_FLOAT_BASE + f.n_local_float;
                f.n_local_float += 1;
                a
            }
            Type::Nula => unreachable!("parámetro de tipo nula"),
        }
    }

    /// Devuelve la dirección de una constante entera, asignándola si no existía.
    pub fn intern_int(&mut self, v: i64) -> u32 {
        if let Some(&a) = self.const_table.int_consts.get(&v) {
            return a;
        }
        let a = self.next_cte_int;
        self.next_cte_int += 1;
        self.const_table.int_consts.insert(v, a);
        a
    }

    /// Devuelve la dirección de una constante flotante, asignándola si no existía.
    pub fn intern_float(&mut self, v: f64) -> u32 {
        let key = v.to_string();
        if let Some(&a) = self.const_table.float_consts.get(&key) {
            return a;
        }
        let a = self.next_cte_float;
        self.next_cte_float += 1;
        self.const_table.float_consts.insert(key, a);
        a
    }

    // ── Impresión ─────────────────────────────────────────────────────────────

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
            gv.sort_by_key(|(_, v)| v.address);
            for (name, info) in gv {
                println!("    {:<16} : {:<10} @ {}", name, info.var_type, info.address);
            }
        }

        // ── Tabla de constantes
        println!("\n  [tabla de constantes]");
        if self.const_table.int_consts.is_empty() && self.const_table.float_consts.is_empty() {
            println!("    (ninguna)");
        } else {
            let mut ic: Vec<_> = self.const_table.int_consts.iter().collect();
            ic.sort_by_key(|(_, a)| **a);
            for (v, a) in ic {
                println!("    {:<10} : entero    @ {}", v, a);
            }
            let mut fc: Vec<_> = self.const_table.float_consts.iter().collect();
            fc.sort_by_key(|(_, a)| **a);
            for (v, a) in fc {
                println!("    {:<10} : flotante  @ {}", v, a);
            }
        }

        // ── Funciones
        let mut funcs: Vec<_> = self.funcs.iter().collect();
        funcs.sort_by_key(|(k, _)| k.as_str());
        for (fname, info) in funcs {
            let params_str = info.params.iter()
                .map(|(n, t)| {
                    let addr = info.local_vars.get(n)
                        .map(|v| v.address.to_string())
                        .unwrap_or_else(|| "?".into());
                    format!("{}: {} @ {}", n, t, addr)
                })
                .collect::<Vec<_>>()
                .join(", ");
            println!("\n  [función '{}']", fname);
            println!("  Retorna   : {}", info.return_type);
            println!("  Parámetros: ({})", params_str);
            println!(
                "  Recursos  : locals(int={}, float={}) temps(int={}, float={}) start_quad={}",
                info.n_local_int, info.n_local_float,
                info.n_temp_int,  info.n_temp_float,
                info.start_quad
            );

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
                locals.sort_by_key(|(_, v)| v.address);
                for (n, v) in locals {
                    println!("    {:<16} : {:<10} @ {}", n, v.var_type, v.address);
                }
            }
        }
        println!();
    }
}
