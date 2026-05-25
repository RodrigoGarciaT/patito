use crate::types::Type;

// ── Operadores del lenguaje ───────────────────────────────────────────────────
//
// El enum Op cubre TODO lo que puede aparecer en un cuádruplo, además de los
// operadores que entran a la pila durante el análisis de expresiones.
//
// Categorías:
//   · Aritméticos y relacionales — binarios, entran a la pila de operadores
//   · Asig                       — asignación, entra a la pila (precedencia 1)
//   · LParen                     — "fondo falso" para paréntesis de agrupación
//   · Print / PrintStr / Return  — solo aparecen en cuádruplos, nunca en pila
//   · Goto / GotoF               — saltos para control de flujo (si / mientras)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    // Binarios — entran a la pila de operadores
    Add, Sub, Mul, Div,
    Gt,  Lt,  Eq,  Ne,
    Asig,
    LParen,

    // Solo aparecen en cuádruplos
    Print, PrintStr, Return,
    Goto, GotoF,
}

impl Op {
    /// Precedencia para el algoritmo de pila.
    /// Reglas: al hacer push del operador `new`, mientras
    /// `top != LParen && top.precedence() >= new.precedence()`, colapsa.
    /// Esto hace que `(` (prec 0) nunca se colapse por precedencia y que
    /// `=` (prec 1) se quede en el fondo hasta el force-collapse del `;`.
    pub fn precedence(self) -> u8 {
        match self {
            Op::LParen => 0,
            Op::Asig   => 1,
            Op::Gt | Op::Lt | Op::Eq | Op::Ne => 2,
            Op::Add | Op::Sub => 3,
            Op::Mul | Op::Div => 4,
            // No-binarios: no entran a la pila; no se comparan con nadie.
            Op::Print | Op::PrintStr | Op::Return | Op::Goto | Op::GotoF => 0,
        }
    }

    /// Iterador de operadores binarios para imprimir el cubo.
    pub fn binary_all() -> &'static [Op] {
        &[
            Op::Add, Op::Sub, Op::Mul, Op::Div,
            Op::Gt,  Op::Lt,  Op::Ne,  Op::Eq,
        ]
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Op::Add      => "+",
            Op::Sub      => "-",
            Op::Mul      => "*",
            Op::Div      => "/",
            Op::Gt       => ">",
            Op::Lt       => "<",
            Op::Ne       => "!=",
            Op::Eq       => "==",
            Op::Asig     => "=",
            Op::LParen   => "(",
            Op::Print    => "PRINT",
            Op::PrintStr => "PRINT_STR",
            Op::Return   => "RETURN",
            Op::Goto     => "GOTO",
            Op::GotoF    => "GOTOF",
        })
    }
}

// ── Cubo semántico ────────────────────────────────────────────────────────────
//
// result_type(left, op, right) → Some(resultado) o None si hay error de tipos.
//
// Reglas de Patito:
//   · Aritméticos (+, -, *, /): entero op entero → entero
//                                entero op flotante (o viceversa) → flotante
//   · Relacionales (>, <, !=, ==): siempre → entero (Patito no tiene bool)
//   · nula nunca puede ser operando → None
//   · Operadores no-binarios (Asig, Print, etc.) no van por aquí → None

pub fn result_type(left: Type, op: Op, right: Type) -> Option<Type> {
    if left == Type::Nula || right == Type::Nula {
        return None;
    }
    Some(match op {
        Op::Add | Op::Sub | Op::Mul | Op::Div => {
            match (left, right) {
                (Type::Entero, Type::Entero) => Type::Entero,
                _                            => Type::Flotante,
            }
        }
        Op::Gt | Op::Lt | Op::Ne | Op::Eq => {
            Type::Entero
        }
        _ => return None,
    })
}

// ── Compatibilidad de asignación ──────────────────────────────────────────────
//
// Reglas (permisivas):
//   entero   = entero   → OK
//   flotante = flotante → OK
//   flotante = entero   → OK (ampliación implícita, igual que en el cubo aritmético)
//   entero   = flotante → Error (no se permite estrechamiento)

pub fn is_assignable(to: Type, from: Type) -> bool {
    matches!(
        (to, from),
        (Type::Entero,   Type::Entero)
        | (Type::Flotante, Type::Flotante)
        | (Type::Flotante, Type::Entero)
    )
}

// ── Impresión del cubo como tabla ASCII ───────────────────────────────────────

pub fn print_cube() {
    let numeric = [Type::Entero, Type::Flotante];
    let ops     = Op::binary_all();

    println!("  Tabla de resultados de tipo — (izquierda) OP (derecha)");
    println!("  E = entero  F = flotante  - = error de tipos");
    println!();

    // encabezado de columnas
    print!("  {:<18}", "izquierda \\ OP");
    for op in ops { print!(" {:<3}", format!("{}", op)); }
    println!();
    println!("  {}", "─".repeat(18 + ops.len() * 4));

    for left in numeric {
        for right in numeric {
            print!("  {:<18}", format!("{} op {}", left, right));
            for op in ops {
                let cell = match result_type(left, *op, right) {
                    Some(Type::Entero)   => " E ",
                    Some(Type::Flotante) => " F ",
                    _                    => " - ",
                };
                print!(" {:<3}", cell);
            }
            println!();
        }
        println!("  {}", "─".repeat(18 + ops.len() * 4));
    }

    println!();
    println!("  Compatibilidad de asignación  (variable : TO = FROM)");
    println!();
    print!("  {:<12}", "TO \\ FROM");
    for t in numeric { print!("{:>12}", t.to_string()); }
    println!();
    for to in numeric {
        print!("  {:<12}", to.to_string());
        for from in numeric {
            print!("{:>12}", if is_assignable(to, from) { "OK" } else { "Error" });
        }
        println!();
    }
    println!();
}
