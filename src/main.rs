// Patito — analizador léxico, sintáctico y semántico (Entrega 3)
// Entrega 1: Scanner (logos) + Parser (LALRPOP)
// Entrega 2: Cubo semántico, Directorio de Funciones, Tablas de Variables.
// Entrega 3: Pilas (operadores, operandos, jumps) + Fila de cuádruplos,
//            traducción de expresiones aritméticas/relacionales y de los
//            estatutos lineales del lenguaje.

mod lexer;
mod types;
mod semantic_cube;
mod func_dir;
mod quad_gen;
mod context;
mod vm_payload;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);

use lexer::Lexer;
use context::SemanticContext;

// ── Función de parseo ─────────────────────────────────────────────────────────

fn parse(src: &str) -> Result<SemanticContext, String> {
    let mut ctx = SemanticContext::new();
    let lexer   = Lexer::new(src);
    grammar::ProgramaParser::new()
        .parse(&mut ctx, lexer)
        .map_err(|e| format!("{:?}", e))?;
    Ok(ctx)
}

// ── Helpers de prueba ─────────────────────────────────────────────────────────

fn test_valid(label: &str, src: &str) -> bool {
    match parse(src) {
        Err(e) => {
            println!("  [FAIL] {label}");
            println!("           Error sintáctico: {e}");
            false
        }
        Ok(ctx) if ctx.has_errors() => {
            println!("  [FAIL] {label}  — errores semánticos inesperados:");
            for e in &ctx.errors { println!("           {e}"); }
            false
        }
        Ok(_) => { println!("  [OK]   {label}"); true }
    }
}

fn test_sem_error(label: &str, src: &str, fragment: &str) -> bool {
    match parse(src) {
        Err(e) => {
            println!("  [FAIL] {label}  — error sintáctico inesperado: {e}");
            false
        }
        Ok(ctx) if ctx.errors.iter().any(|e| e.0.contains(fragment)) => {
            println!("  [OK]   {label}  — error semántico detectado correctamente");
            true
        }
        Ok(ctx) if ctx.has_errors() => {
            println!("  [FAIL] {label}  — errores presentes pero ninguno contiene '{fragment}':");
            for e in &ctx.errors { println!("           {e}"); }
            false
        }
        Ok(_) => {
            println!("  [FAIL] {label}  — se esperaba error semántico pero no hubo ninguno");
            false
        }
    }
}

fn test_syntax_error(label: &str, src: &str) -> bool {
    match parse(src) {
        Err(_) => { println!("  [OK]   {label}  — error sintáctico detectado"); true }
        Ok(ctx) if ctx.has_errors() => {
            println!("  [FAIL] {label}  — solo errores semánticos (esperaba sintáctico)");
            false
        }
        Ok(_) => {
            println!("  [FAIL] {label}  — debió rechazarse sintácticamente");
            false
        }
    }
}

/// Muestra la fila de cuádruplos generada por un programa válido.
fn show_quads(label: &str, src: &str) {
    println!();
    println!("─── {label} ───");
    match parse(src) {
        Err(e) => println!("  Error sintáctico: {e}"),
        Ok(ctx) if ctx.has_errors() => {
            println!("  Errores semánticos:");
            for e in &ctx.errors { println!("    {e}"); }
        }
        Ok(ctx) => {
            ctx.qg.print();
            ctx.qg.dump_stacks_if_dirty();
        }
    }
}

// ── Modo "compilar archivo" — Opción A subcomando ────────────────────────────

fn compile_file(path: &str) {
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|e| {
            eprintln!("No pude leer '{}': {}", path, e);
            std::process::exit(1);
        });

    match parse(&source) {
        Ok(ctx) if !ctx.has_errors() => {
            let payload = vm_payload::build_payload(&ctx);
            let json = serde_json::to_string_pretty(&payload)
                .expect("error serializando a JSON");

            // Determinar el path de salida: cambia .patito o .txt por .obj
            let obj_path = if path.ends_with(".patito") {
                path.replace(".patito", ".obj")
            } else if path.ends_with(".txt") {
                path.replace(".txt", ".obj")
            } else {
                format!("{}.obj", path)
            };

            std::fs::write(&obj_path, json)
                .expect("no se pudo escribir el .obj");
            println!("✅ {} → {}", path, obj_path);
        }
        Ok(ctx) => {
            eprintln!("Errores semánticos en '{}':", path);
            for e in &ctx.errors {
                eprintln!("  {}", e);
            }
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error sintáctico en '{}': {}", path, e);
            std::process::exit(1);
        }
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    // Si se pasa un archivo como argumento → modo compilar
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        compile_file(&args[1]);
        return;
    }

    // Si no → modo tests (comportamiento existente)
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Patito — Entrega 3: Cuádruplos");
    println!("═══════════════════════════════════════════════════════════════\n");

    let mut ok    = 0u32;
    let mut total = 0u32;

    macro_rules! v   { ($l:expr, $s:expr) => {{ total+=1; if test_valid($l,$s)              {ok+=1;} }}; }
    macro_rules! sem { ($l:expr, $s:expr, $f:expr) => {{ total+=1; if test_sem_error($l,$s,$f)   {ok+=1;} }}; }
    macro_rules! syn { ($l:expr, $s:expr) => {{ total+=1; if test_syntax_error($l,$s)       {ok+=1;} }}; }

    // ── Programas válidos heredados ───────────────────────────────────────────
    println!("  — Programas válidos —\n");

    v!("T01 — programa mínimo",
        r#"programa hola;
           inicio { escribe("Hola, mundo!"); }
           fin"#
    );

    v!("T02 — vars, asignación, expresiones flotantes",
        r#"programa areaCirculo;
           vars radio : entero; area : flotante;
           inicio {
               radio = 5;
               area  = 3.14159 * radio * radio;
               escribe("Radio: ", radio);
               escribe("Area: ", area);
           }
           fin"#
    );

    v!("T03 — condicional si/sino",
        r#"programa mayor;
           vars x, y : entero;
           inicio {
               x = 15; y = 8;
               si (x > y) { escribe(x); } sino { escribe(y); };
           }
           fin"#
    );

    v!("T04 — ciclo mientras/haz (factorial)",
        r#"programa factorial;
           vars n, fact, i : entero;
           inicio {
               n = 5; fact = 1; i = 1;
               mientras (i < n) haz { i = i + 1; fact = fact * i; };
               escribe("Factorial: ", fact);
           }
           fin"#
    );

    // T05 reescrito para Entrega 3 — las llamadas a función dentro de
    // expresiones se cubren en Entrega 4 (ERA/PARAM/GOSUB). Aquí solo
    // se declaran y se valida que el regresa cuadre con el return_type.
    v!("T05 — funciones independientes con regresa válido",
        r#"programa funcs;
           vars resultado : entero;

           entero cuadrado(n : entero) {
               { regresa(n * n); }
           };

           entero suma(x : entero, y : entero) {
               vars temp : entero;
               { temp = x + y; regresa(temp); }
           };

           flotante mitad(z : entero) {
               { regresa(z); }
           };

           inicio {
               resultado = 42;
               escribe("ok: ", resultado);
           }
           fin"#
    );

    v!("T06 — Fibonacci (múltiples ids en una declaración)",
        r#"programa fibonacci;
           vars n, a, b, temp, i : entero;
           inicio {
               n = 10; a = 0; b = 1; i = 0;
               mientras (i < n) haz {
                   escribe(a);
                   temp = a + b; a = b; b = temp; i = i + 1;
               };
           }
           fin"#
    );

    // ── Errores sintácticos ───────────────────────────────────────────────────
    println!();
    println!("  — Errores sintácticos esperados —\n");

    syn!("T07 — falta id de programa",    "programa;");
    syn!("T08 — falta ; en asignación",  "programa t; inicio { x = 1 } fin");
    syn!("T09 — falta ; al final del si",
         "programa t; inicio { si (x > 0) { escribe(x); } } fin");

    // ── Errores semánticos ────────────────────────────────────────────────────
    println!();
    println!("  — Errores semánticos esperados —\n");

    sem!("T10 — variable global doblemente declarada",
        r#"programa test;
           vars x : entero; x : flotante;
           inicio { escribe(x); }
           fin"#,
        "x"
    );

    sem!("T11 — variable local doblemente declarada",
        r#"programa test;
           entero foo(n : entero) {
               vars k : entero; k : flotante;
               { regresa(k); }
           };
           inicio { escribe(foo(1)); }
           fin"#,
        "k"
    );

    sem!("T12 — función doblemente declarada",
        r#"programa test;
           entero doble(n : entero) { { regresa(n + n); } };
           entero doble(m : entero) { { regresa(m * 2); } };
           inicio { escribe(doble(3)); }
           fin"#,
        "doble"
    );

    sem!("T13 — parámetro duplicado en función",
        r#"programa test;
           entero suma(a : entero, a : entero) { { regresa(a + a); } };
           inicio { escribe(suma(1, 2)); }
           fin"#,
        "a"
    );

    v!("T14 — mismo nombre de var en scopes distintos",
        r#"programa test;
           vars k : entero;
           entero foo(k : entero) { { regresa(k + 1); } };
           inicio { k = 5; escribe(foo(k)); }
           fin"#
    );

    // ── Nuevos errores semánticos de Entrega 3 ────────────────────────────────
    println!();
    println!("  — Errores semánticos nuevos (Entrega 3) —\n");

    sem!("T15 — uso de variable no declarada",
        r#"programa test;
           inicio { x = 1; }
           fin"#,
        "no declarada"
    );

    sem!("T16 — asignación incompatible (entero = flotante)",
        r#"programa test;
           vars x : entero;
           inicio { x = 3.14; }
           fin"#,
        "incompatible"
    );

    sem!("T17 — condición no entera",
        r#"programa test;
           vars x : flotante;
           inicio { si (x + 1.0) { escribe(x); }; }
           fin"#,
        "condición"
    );

    // ── Resumen ───────────────────────────────────────────────────────────────
    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Resultado: {ok}/{total} pruebas correctas");
    println!("═══════════════════════════════════════════════════════════════");

    // ── Cuádruplos para programas de prueba ───────────────────────────────────
    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Fila de cuádruplos — programas de prueba");
    println!("═══════════════════════════════════════════════════════════════");

    show_quads("Q01 — asignación con expresión mixta (precedencia)",
        r#"programa q01;
           vars a, b, c, x : entero;
           inicio {
               a = 2; b = 3; c = 4;
               x = a + b * c;
           }
           fin"#
    );

    show_quads("Q02 — paréntesis cambian precedencia",
        r#"programa q02;
           vars a, b, c, x : entero;
           inicio {
               a = 2; b = 3; c = 4;
               x = (a + b) * c;
           }
           fin"#
    );

    show_quads("Q03 — área del círculo (mezcla entero/flotante)",
        r#"programa q03;
           vars radio : entero; area : flotante;
           inicio {
               radio = 5;
               area  = 3.14159 * radio * radio;
               escribe("Radio: ", radio);
               escribe("Area: ", area);
           }
           fin"#
    );

    show_quads("Q04 — si/sino (mayor de dos)",
        r#"programa q04;
           vars x, y : entero;
           inicio {
               x = 15; y = 8;
               si (x > y) { escribe(x); } sino { escribe(y); };
           }
           fin"#
    );

    show_quads("Q05 — si sin sino",
        r#"programa q05;
           vars x : entero;
           inicio {
               x = 10;
               si (x > 0) { escribe("positivo"); };
           }
           fin"#
    );

    show_quads("Q06 — while (factorial)",
        r#"programa q06;
           vars n, fact, i : entero;
           inicio {
               n = 5; fact = 1; i = 1;
               mientras (i < n) haz { i = i + 1; fact = fact * i; };
               escribe("Factorial: ", fact);
           }
           fin"#
    );

    show_quads("Q07 — Fibonacci (while + múltiples asignaciones)",
        r#"programa q07;
           vars n, a, b, temp, i : entero;
           inicio {
               n = 10; a = 0; b = 1; i = 0;
               mientras (i < n) haz {
                   escribe(a);
                   temp = a + b; a = b; b = temp; i = i + 1;
               };
           }
           fin"#
    );

    show_quads("Q08 — si anidado dentro de while",
        r#"programa q08;
           vars i, par : entero;
           inicio {
               i = 0; par = 0;
               mientras (i < 10) haz {
                   si (i < 5) { par = par + i; } sino { par = par - i; };
                   i = i + 1;
               };
               escribe("Resultado: ", par);
           }
           fin"#
    );

    show_quads("Q09 — relacionales y aritmética combinadas",
        r#"programa q09;
           vars a, b, c, r : entero;
           inicio {
               a = 1; b = 2; c = 3;
               r = a + b * c == c + b;
               escribe(r);
           }
           fin"#
    );

    show_quads("Q10 — regresa con cálculo",
        r#"programa q10;
           entero cuadrado(n : entero) {
               { regresa(n * n); }
           };
           inicio {
               escribe("cuadrado de 7");
           }
           fin"#
    );

    // ── Stress test integral ─────────────────────────────────────────────────
    // Q11 combina TODO lo que la entrega exige en un solo programa:
    //   · expresiones aritméticas con precedencia mixta
    //   · expresiones relacionales (>, <, ==, !=)
    //   · asignación entre tipos compatibles (flotante = entero, etc.)
    //   · escribe con letreros y expresiones mezclados
    //   · si sin sino
    //   · si/sino simple
    //   · si/sino encadenado (else-if pattern)
    //   · mientras anidado dentro de mientras
    //   · si anidado dentro de mientras
    //   · uso de la misma variable en varios scopes de control

    show_quads("Q11 — STRESS TEST: ciclos anidados, ifs encadenados, mezcla de tipos",
        r#"programa stressTest;
           vars n, i, j, suma, prod, max, min, contador : entero;
                promedio, escala : flotante;
           inicio {
               n = 10;
               i = 1;
               suma = 0;
               prod = 1;
               max = 0;
               min = 100;
               contador = 0;
               escala = 1.5;

               mientras (i < n) haz {
                   suma = suma + i;
                   prod = prod * i;

                   si (i > max) {
                       max = i;
                   };

                   si (i < min) {
                       min = i;
                   } sino {
                       contador = contador + 1;
                   };

                   j = 0;
                   mientras (j < i) haz {
                       suma = suma + 1;
                       j = j + 1;
                   };

                   i = i + 1;
               };

               promedio = suma * escala;

               si (suma > prod) {
                   escribe("suma mayor: ", suma);
               } sino {
                   si (suma == prod) {
                       escribe("iguales");
                   } sino {
                       escribe("prod mayor: ", prod);
                   };
               };

               escribe("max: ", max);
               escribe("min: ", min);
               escribe("contador: ", contador);
               escribe("promedio: ", promedio);
           }
           fin"#
    );

    // ── Cuádruplos de funciones (Entrega 4 Parte 2) ──────────────────────────
    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Fila de cuádruplos — funciones (Entrega 4 Parte 2)");
    println!("═══════════════════════════════════════════════════════════════");

    // QF01 — llamada como factor con valor de retorno usado en expresión.
    // Cubre: GOTO inicial, ERA/PARAM/GOSUB, RETURN con return_addr,
    // copy-to-temp del retorno, dos llamadas a la misma función dentro de
    // una expresión (evita pisar return_addr gracias al copy-to-temp).
    show_quads("QF01 — doble(3) + doble(4) (call como factor)",
        r#"programa fcalls;
           entero doble(n : entero) {
               { regresa(n + n); }
           };
           inicio {
               escribe(doble(3) + doble(4));
           }
           fin"#
    );

    // QF02 — llamada como sentencia + función nula.
    // Cubre: ENDFUNC de función nula, ERA/GOSUB sin valor de retorno,
    // CallStmt en vez de FactorAtomCall.
    show_quads("QF02 — saluda() como sentencia (función nula)",
        r#"programa fcalls_void;
           nula saluda() {
               { escribe("hola"); }
           };
           inicio {
               saluda();
               escribe("fin");
           }
           fin"#
    );

    // QF03 — llamada con varios argumentos y aritmética dentro del cuerpo.
    // Cubre: PARAM en orden por cada arg, validación de tipo de cada arg,
    // promoción a flotante dentro del cuerpo.
    show_quads("QF03 — suma(2, 3) usada en asignación",
        r#"programa fcalls_args;
           vars resultado : entero;
           entero suma(a : entero, b : entero) {
               vars t : entero;
               { t = a + b; regresa(t); }
           };
           inicio {
               resultado = suma(2, 3);
               escribe(resultado);
           }
           fin"#
    );

    // QF04 — llamada anidada: foo(bar(x), y) — la razón por la que el
    // call_stack debe ser pila y no contador único.
    show_quads("QF04 — llamada anidada cuadrado(cuadrado(3))",
        r#"programa fcalls_nested;
           entero cuadrado(n : entero) {
               { regresa(n * n); }
           };
           inicio {
               escribe(cuadrado(cuadrado(3)));
           }
           fin"#
    );

    // ── Directorio de Funciones (T05 — calculadora) ───────────────────────────
    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Directorio de Funciones — calculadora");
    println!("═══════════════════════════════════════════════════════════════\n");

    let t05 = r#"programa calculadora;
        vars a, b : entero;

        entero cuadrado(n : entero) {
            { regresa(n * n); }
        };

        entero potenciaCuatro(n : entero) {
            vars cuad : entero;
            { regresa(cuad); }
        };

        inicio {
            a = 3; b = 4;
            escribe(a, " y ", b);
        }
        fin"#;

    if let Ok(ctx) = parse(t05) {
        ctx.func_dir.print();
    }

    // ── Cubo Semántico ────────────────────────────────────────────────────────
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Cubo Semántico");
    println!("═══════════════════════════════════════════════════════════════\n");
    semantic_cube::print_cube();

    if ok < total {
        std::process::exit(1);
    }
}
