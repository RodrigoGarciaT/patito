// Capa de exportación: convierte el estado interno del compilador
// (SemanticContext) en un VmPayload listo para serializar a JSON y
// pasárselo a la VM en C++.

use serde::Serialize;
use std::collections::HashMap;

use crate::context::SemanticContext;
use crate::semantic_cube::Op;
use crate::types::{GLOBAL_INT_BASE, GLOBAL_FLOAT_BASE};

// ── Structs paralelas (paralelas a las del C++) ──────────────────────────────

#[derive(Serialize)]
pub struct VmQuad {
    pub op:     String,
    pub arg1:   String,
    pub arg2:   String,
    pub result: String,
}

#[derive(Serialize)]
pub struct VmConstants {
    pub ints:   HashMap<u32, i64>,   // dirección → valor
    pub floats: HashMap<u32, f64>,
}

#[derive(Serialize)]
pub struct GlobalSizes {
    pub n_int:   u32,
    pub n_float: u32,
}

#[derive(Serialize)]
pub struct FuncMeta {
    pub n_local_int:   u32,
    pub n_local_float: u32,
    pub n_temp_int:    u32,
    pub n_temp_float:  u32,
}

#[derive(Serialize)]
pub struct MainMeta {
    pub n_temp_int:   u32,
    pub n_temp_float: u32,
}

#[derive(Serialize)]
pub struct VmPayload {
    pub quads:     Vec<VmQuad>,
    pub constants: VmConstants,
    pub globals:   GlobalSizes,
    pub funcs:     HashMap<String, FuncMeta>,
    pub main:      MainMeta,
}

// ── Conversión de Op a string ────────────────────────────────────────────────
// La VM en C++ matchea contra estos strings exactos en su parse_op().

fn op_to_string(op: Op) -> String {
    match op {
        Op::Add      => "+",
        Op::Sub      => "-",
        Op::Mul      => "*",
        Op::Div      => "/",
        Op::Gt       => ">",
        Op::Lt       => "<",
        Op::Eq       => "==",
        Op::Ne       => "!=",
        Op::Asig     => "=",
        Op::Print    => "PRINT",
        Op::PrintStr => "PRINT_STR",
        Op::Return   => "RETURN",
        Op::Goto     => "GOTO",
        Op::GotoF    => "GOTOF",
        Op::Era      => "ERA",
        Op::Param    => "PARAM",
        Op::Gosub    => "GOSUB",
        Op::EndFunc  => "ENDFUNC",
        // LParen nunca debe aparecer en cuádruplos emitidos — solo en la pila
        Op::LParen   => unreachable!("LParen no debe llegar a un cuádruplo"),
    }.to_string()
}

// ── Función principal: construir el payload desde el contexto ────────────────

pub fn build_payload(ctx: &SemanticContext) -> VmPayload {
    // 1. Cuádruplos: convertir cada Quadruple a VmQuad (cambia Op enum a string)
    let quads: Vec<VmQuad> = ctx.qg.quads.iter().map(|q| VmQuad {
        op:     op_to_string(q.op),
        arg1:   q.arg1.clone(),
        arg2:   q.arg2.clone(),
        result: q.result.clone(),
    }).collect();

    // 2. Constantes: invertir HashMap (valor → dir) a (dir → valor)
    let mut const_ints: HashMap<u32, i64> = HashMap::new();
    for (val, addr) in &ctx.func_dir.const_table.int_consts {
        const_ints.insert(*addr, *val);
    }
    let mut const_floats: HashMap<u32, f64> = HashMap::new();
    for (key_str, addr) in &ctx.func_dir.const_table.float_consts {
        let val: f64 = key_str.parse()
            .expect("constante flotante inválida en const_table");
        const_floats.insert(*addr, val);
    }

    // 3. Globales: restar las bases para obtener el conteo real
    let globals = GlobalSizes {
        n_int:   ctx.func_dir.next_global_int   - GLOBAL_INT_BASE,
        n_float: ctx.func_dir.next_global_float - GLOBAL_FLOAT_BASE,
    };

    // 4. Funciones: snapshot de los contadores de cada FuncInfo
    let funcs: HashMap<String, FuncMeta> = ctx.func_dir.funcs.iter()
        .map(|(name, info)| {
            (name.clone(), FuncMeta {
                n_local_int:   info.n_local_int,
                n_local_float: info.n_local_float,
                n_temp_int:    info.n_temp_int,
                n_temp_float:  info.n_temp_float,
            })
        }).collect();

    // 5. Main: los contadores finales del QuadGen son los de main (Opción A)
    let (main_ti, main_tf) = ctx.qg.take_temp_counts();
    let main = MainMeta {
        n_temp_int:   main_ti,
        n_temp_float: main_tf,
    };

    VmPayload {
        quads,
        constants: VmConstants {
            ints:   const_ints,
            floats: const_floats,
        },
        globals,
        funcs,
        main,
    }
}
