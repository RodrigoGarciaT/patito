// Patito — analizador léxico y sintáctico 
// Usa `logos` para el lexer y `lalrpop` para el parser.

use std::panic;

mod lexer;
mod semantic;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);

use lexer::Lexer;

fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    match payload.downcast::<String>() {
        Ok(message) => *message,
        Err(payload) => match payload.downcast::<&'static str>() {
            Ok(message) => (*message).to_string(),
            Err(_) => "panic no textual".to_string(),
        },
    }
}

/// Intenta parsear `src` como un programa Patito.
/// Devuelve Ok(()) si es valido, o un mensaje de error.
/// Atrapa panics y los convierte en Err.
fn parse(src: &str) -> Result<(), String> {
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let mut sem = semantic::SemanticState::nuevo();
        let lexer = Lexer::new(src);
        grammar::ProgramaParser::new()
            .parse(&mut sem, lexer)
            .map_err(|e| format!("{:?}", e))
    }));

    match result {
        Ok(parse_result) => parse_result,
        Err(payload) => Err(panic_payload_to_string(payload)),
    }
}

/// Imprime el resultado del test y devuelve true si pasó.
fn test(label: &str, src: &str) -> bool {
    match parse(src) {
        Ok(_) => {
            println!("  [OK]   {}", label);
            true
        }
        Err(e) => {
            println!("  [FAIL] {}  →  {}", label, e);
            false
        }
    }
}

/// Imprime resultado cuando se espera que falle; devuelve true si el error ocurre.
fn test_should_fail(label: &str, src: &str) -> bool {
    match parse(src) {
        Ok(_) => {
            println!("  [FAIL] {} (debió rechazarse)", label);
            false
        }
        Err(e) => {
            println!("  [OK]   {}  →  {}", label, e);
            true
        }
    }
}

fn main() {
    panic::set_hook(Box::new(|_| {}));
    println!("═══════════════════════════════════════════════════════");
    println!("  Patito — pruebas léxico/sintácticas");
    println!("═══════════════════════════════════════════════════════\n");

    let mut ok = 0u32;
    let mut total = 0u32;

    macro_rules! run {
        ($label:expr, $src:expr) => {{
            total += 1;
            if test($label, $src) { ok += 1; }
        }};
    }

    macro_rules! run_err {
        ($label:expr, $src:expr) => {{
            total += 1;
            if test_should_fail($label, $src) { ok += 1; }
        }};
    }

    // ── Ejemplo 1: Programa mínimo ("Hola, mundo!") ───────────────────────────
    run!(
        "Ejemplo 1 — programa mínimo",
        r#"
        programa hola;
        inicio
        {
            escribe("Hola, mundo!");
        }
        fin
        "#
    );

    // ── Ejemplo 2: Cálculo del área de un círculo ─────────────────────────────
    run!(
        "Ejemplo 2 — vars, asignación, expresiones flotantes",
        r#"
        programa areaCirculo;
        vars
            radio : entero;
            area  : flotante;
        inicio
        {
            radio = 5;
            area = 3.14159 * radio * radio;
            escribe("Radio: ", radio);
            escribe("Area aproximada: ", area);
        }
        fin
        "#
    );

    // ── Ejemplo 3: Mayor de dos números ──────────────────────────────────────
    run!(
        "Ejemplo 3 — condicional si/sino",
        r#"
        programa mayor;
        vars
            x, y : entero;
        inicio
        {
            x = 15;
            y = 8;
            si (x > y) {
                escribe("x es el mayor con valor ", x);
            } sino {
                escribe("y es el mayor o igual con valor ", y);
            };
        }
        fin
        "#
    );

    // ── Ejemplo 4: Factorial iterativo ───────────────────────────────────────
    run!(
        "Ejemplo 4 — ciclo mientras/haz",
        r#"
        programa factorial;
        vars
            n, fact, i : entero;
        inicio
        {
            n    = 5;
            fact = 1;
            i    = 1;
            mientras (i < n) haz {
                i    = i + 1;
                fact = fact * i;
            };
            escribe("El factorial de ", n, " es ", fact);
        }
        fin
        "#
    );

    // ── Ejemplo 5: Funciones con valor de retorno ─────────────────────────────
    // Las funciones usan dos pares de llaves: las externas son parte de la
    // regla FUNCS y las internas corresponden al CUERPO.
    run!(
        "Ejemplo 5 — funciones con regresa y llamadas anidadas",
        r#"
        programa calculadora;
        vars
            a, b : entero;

        entero cuadrado(n : entero) {
            {
                regresa(n * n);
            }
        };

        entero potenciaCuatro(n : entero) {
            vars
                cuad : entero;
            {
                cuad = cuadrado(n);
                regresa(cuadrado(cuad));
            }
        };

        inicio
        {
            a = 3;
            b = potenciaCuatro(a);
            escribe(a, " a la cuarta potencia es ", b);
        }
        fin
        "#
    );

    // ── Ejemplo 6: Serie de Fibonacci ────────────────────────────────────────
    run!(
        "Ejemplo 6 — Fibonacci (múltiples vars en una declaración)",
        r#"
        programa fibonacci;
        vars
            n, a, b, temp, i : entero;
        inicio
        {
            n = 10;
            a = 0;
            b = 1;
            i = 0;
            escribe("Primeros ", n, " numeros de Fibonacci:");
            mientras (i < n) haz {
                escribe(a);
                temp = a + b;
                a    = b;
                b    = temp;
                i    = i + 1;
            };
        }
        fin
        "#
    );

    // ── Casos de error (deben fallar) ─────────────────────────────────────────
    println!();
    println!("  — Errores de sintaxis —");

    total += 1;
    let falla = parse("programa;").is_err(); // falta el id del programa
    if falla {
        println!("  [OK]   Error esperado: falta id de programa");
        ok += 1;
    } else {
        println!("  [FAIL] Debió rechazar 'programa;' sin id");
    }

    total += 1;
    let falla = parse(r#"
        programa test;
        inicio
        {
            x = 1
        }
        fin
    "#).is_err(); // falta el ; después de la asignación
    if falla {
        println!("  [OK]   Error esperado: falta ; en asignación");
        ok += 1;
    } else {
        println!("  [FAIL] Debió rechazar asignación sin ;");
    }

    total += 1;
    let falla = parse(r#"
        programa test;
        inicio
        {
            si (x > 0) {
                escribe(x);
            }
        }
        fin
    "#).is_err(); // falta el ; al final del si
    if falla {
        println!("  [OK]   Error esperado: falta ; al final del si");
        ok += 1;
    } else {
        println!("  [FAIL] Debió rechazar si sin ;");
    }

    println!();
    println!("  — Errores semánticos —");

    run_err!(
        "SemError 1 — variable duplicada",
        r#"
        programa dupvar;
        vars
            x : entero;
            x : entero;
        inicio
        {
        }
        fin
        "#
    );

    run_err!(
        "SemError 2 — asignacion a variable no declarada",
        r#"
        programa nodcl;
        inicio
        {
            y = 10;
        }
        fin
        "#
    );

    run_err!(
        "SemError 3 — asignacion flotante a entero",
        r#"
        programa badassign;
        vars
            a : entero;
        inicio
        {
            a = 3.14;
        }
        fin
        "#
    );

    run_err!(
        "SemError 4 — funcion duplicada",
        r#"
        programa dupfunc;

        entero foo(x : entero) {
            {
                regresa(x);
            }
        };

        entero foo(y : entero) {
            {
                regresa(y);
            }
        };

        inicio
        {
        }
        fin
        "#
    );

    run_err!(
        "SemError 5 — llamada a funcion no declarada",
        r#"
        programa noexist;
        inicio
        {
            foo(5);
        }
        fin
        "#
    );

    run_err!(
        "SemError 6 — regresa fuera de funcion (en global)",
        r#"
        programa regresa_global;
        inicio
        {
            regresa(42);
        }
        fin
        "#
    );

    run_err!(
        "SemError 7 — regresa con tipo incompatible",
        r#"
        programa rettype;

        entero suma(a : entero, b : entero) {
            {
                regresa(3.14);
            }
        };

        inicio
        {
        }
        fin
        "#
    );

    run_err!(
        "SemError 8 — regresa en funcion con retorno nula",
        r#"
        programa regresa_nula;

        nula doNothing(x : entero) {
            {
                regresa(x);
            }
        };

        inicio
        {
        }
        fin
        "#
    );

    println!();
    println!("═══════════════════════════════════════════════════════");
    println!("  Resultado: {}/{} pruebas correctas", ok, total);
    println!("═══════════════════════════════════════════════════════");

    if ok < total {
        std::process::exit(1);
    }
}
