// Patito — analizador léxico y sintáctico 
// Usa `logos` para el lexer y `lalrpop` para el parser.

mod lexer;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);

use lexer::Lexer;

/// Intenta parsear `src` como un programa Patito.
/// Devuelve Ok(()) si es valido, o un mensaje de error.
fn parse(src: &str) -> Result<(), String> {
    let lexer = Lexer::new(src);
    grammar::ProgramaParser::new()
        .parse(lexer)
        .map_err(|e| format!("{:?}", e))
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

fn main() {
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
    println!("  — Casos que deben reportar error —");

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
    println!("═══════════════════════════════════════════════════════");
    println!("  Resultado: {}/{} pruebas correctas", ok, total);
    println!("═══════════════════════════════════════════════════════");

    if ok < total {
        std::process::exit(1);
    }
}
