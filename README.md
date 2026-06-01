# Patito — Compilador

Compilador del lenguaje **Patito** escrito en Rust. Lexer con [logos](https://github.com/maciejhirsz/logos), parser con [LALRPOP](https://github.com/lalrpop/lalrpop), análisis semántico con cubo + directorio de funciones, y generación de código intermedio en forma de cuádruplos con direcciones de memoria virtual.

## Entregas implementadas

- **Entrega 1:** Scanner (logos) + Parser (LALRPOP).
- **Entrega 2:** Cubo semántico + Directorio de Funciones + Tablas de Variables.
- **Entrega 3:** Generación de cuádruplos para estatutos lineales y control de flujo (si/sino, mientras, escribe, regresa) con pilas de operadores/operandos/jumps y back-patching.
- **Entrega 4 — Parte 1:** Traducción de variables, constantes y temporales a direcciones de memoria virtual + tabla de constantes con deduplicación.
- **Entrega 4 — Parte 2:** Cuádruplos de declaración e invocación de funciones (`ERA`, `PARAM`, `GOSUB`, `ENDFUNC`, `RETURN` con `return_addr`) + GOTO inicial del programa.

## Distribución de memoria virtual

Cada segmento tiene 1000 direcciones reservadas:

| Segmento     | Entero | Flotante |
|--------------|--------|----------|
| Globales     | 1000   | 2000     |
| Locales      | 5000   | 6000     |
| Temporales   | 13000  | 14000    |
| Constantes   | 18000  | 19000    |

Las globales y constantes persisten todo el programa. Las locales y temporales se reinician al entrar a cada función (viven en el activation record).

## Cómo correr el programa

1. Instala Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Verifica la instalación:

```bash
cargo --version
rustc --version
```

3. Entra a la carpeta del proyecto:

```bash
cd patito
```

4. Ejecuta el programa:

```bash
cargo run
```

Si sale "command not found: cargo", ejecuta esto y vuelve a intentarlo:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

## Salida esperada

### 1. Validación semántica — 17 casos

```text
═══════════════════════════════════════════════════════════════
  Patito — Entrega 3: Cuádruplos
═══════════════════════════════════════════════════════════════

  — Programas válidos —

  [OK]   T01 — programa mínimo
  [OK]   T02 — vars, asignación, expresiones flotantes
  [OK]   T03 — condicional si/sino
  [OK]   T04 — ciclo mientras/haz (factorial)
  [OK]   T05 — funciones independientes con regresa válido
  [OK]   T06 — Fibonacci (múltiples ids en una declaración)

  — Errores sintácticos esperados —

  [OK]   T07 — falta id de programa
  [OK]   T08 — falta ; en asignación
  [OK]   T09 — falta ; al final del si

  — Errores semánticos esperados —

  [OK]   T10 — variable global doblemente declarada
  [OK]   T11 — variable local doblemente declarada
  [OK]   T12 — función doblemente declarada
  [OK]   T13 — parámetro duplicado en función
  [OK]   T14 — mismo nombre de var en scopes distintos

  — Errores semánticos nuevos (Entrega 3) —

  [OK]   T15 — uso de variable no declarada
  [OK]   T16 — asignación incompatible (entero = flotante)
  [OK]   T17 — condición no entera

═══════════════════════════════════════════════════════════════
  Resultado: 17/17 pruebas correctas
═══════════════════════════════════════════════════════════════
```

### 2. Fila de cuádruplos — programas de prueba

Formato: `índice: (operación, arg1, arg2, resultado)`. Los campos vacíos se imprimen como `_`. Los argumentos son direcciones numéricas: `1000-1999` son globales enteras, `5000-5999` locales enteras, `13000-13999` temporales enteras, `18000-18999` constantes enteras (similar para flotantes con base +1000). El cuádruplo 0 siempre es un `GOTO` que brinca sobre los cuerpos de las funciones hasta el bloque `inicio`.

```text
─── Q01 — asignación con expresión mixta (precedencia) ───
    0: (GOTO     , _     , _     , 1     )
    1: (=        , 18000 , _     , 1000  )
    2: (=        , 18001 , _     , 1001  )
    3: (=        , 18002 , _     , 1002  )
    4: (*        , 1001  , 1002  , 13000 )
    5: (+        , 1000  , 13000 , 13001 )
    6: (=        , 13001 , _     , 1003  )

─── Q02 — paréntesis cambian precedencia ───
    0: (GOTO     , _     , _     , 1     )
    1: (=        , 18000 , _     , 1000  )
    2: (=        , 18001 , _     , 1001  )
    3: (=        , 18002 , _     , 1002  )
    4: (+        , 1000  , 1001  , 13000 )
    5: (*        , 13000 , 1002  , 13001 )
    6: (=        , 13001 , _     , 1003  )

─── Q03 — área del círculo (mezcla entero/flotante) ───
    0: (GOTO     , _     , _     , 1     )
    1: (=        , 18000 , _     , 1000  )
    2: (*        , 19000 , 1000  , 14000 )
    3: (*        , 14000 , 1000  , 14001 )
    4: (=        , 14001 , _     , 2000  )
    5: (PRINT_STR, "Radio: ", _     , _     )
    6: (PRINT    , 1000  , _     , _     )
    7: (PRINT_STR, "Area: ", _     , _     )
    8: (PRINT    , 2000  , _     , _     )

─── Q04 — si/sino (mayor de dos) ───
    0: (GOTO     , _     , _     , 1     )
    1: (=        , 18000 , _     , 1000  )
    2: (=        , 18001 , _     , 1001  )
    3: (>        , 1000  , 1001  , 13000 )
    4: (GOTOF    , 13000 , _     , 7     )
    5: (PRINT    , 1000  , _     , _     )
    6: (GOTO     , _     , _     , 8     )
    7: (PRINT    , 1001  , _     , _     )

─── Q05 — si sin sino ───
    0: (GOTO     , _     , _     , 1     )
    1: (=        , 18000 , _     , 1000  )
    2: (>        , 1000  , 18001 , 13000 )
    3: (GOTOF    , 13000 , _     , 5     )
    4: (PRINT_STR, "positivo", _     , _     )

─── Q06 — while (factorial) ───
    0: (GOTO     , _     , _     , 1     )
    1: (=        , 18000 , _     , 1000  )
    2: (=        , 18001 , _     , 1001  )
    3: (=        , 18001 , _     , 1002  )
    4: (<        , 1002  , 1000  , 13000 )
    5: (GOTOF    , 13000 , _     , 11    )
    6: (+        , 1002  , 18001 , 13001 )
    7: (=        , 13001 , _     , 1002  )
    8: (*        , 1001  , 1002  , 13002 )
    9: (=        , 13002 , _     , 1001  )
   10: (GOTO     , _     , _     , 4     )
   11: (PRINT_STR, "Factorial: ", _     , _     )
   12: (PRINT    , 1001  , _     , _     )

─── Q10 — regresa con cálculo ───
    0: (GOTO     , _     , _     , 4     )
    1: (*        , 5000  , 5000  , 13000 )
    2: (RETURN   , 13000 , _     , 1000  )
    3: (ENDFUNC  , _     , _     , _     )
    4: (PRINT_STR, "cuadrado de 7", _     , _     )

─── Q11 — STRESS TEST: ciclos anidados, ifs encadenados, mezcla de tipos ───
    0: (GOTO     , _     , _     , 1     )
    1: (=        , 18000 , _     , 1000  )
    ...
   55: (PRINT    , 2000  , _     , _     )
```

(Q07, Q08, Q09, Q11 completos se imprimen al ejecutar — están omitidos del README por brevedad.)

### 3. Cuádruplos de funciones (Entrega 4 Parte 2)

```text
─── QF01 — doble(3) + doble(4) (call como factor) ───
    0: (GOTO     , _     , _     , 4     )    ; brinca a inicio
    1: (+        , 5000  , 5000  , 13000 )    ; doble: n + n
    2: (RETURN   , 13000 , _     , 1000  )    ; escribe a return_addr
    3: (ENDFUNC  , _     , _     , _     )
    4: (ERA      , doble , _     , _     )    ; primer doble(3)
    5: (PARAM    , 18000 , _     , 5000  )    ; 3 → param n
    6: (GOSUB    , doble , _     , 1     )
    7: (=        , 1000  , _     , 13001 )    ; copy retorno a temp
    8: (ERA      , doble , _     , _     )    ; segundo doble(4)
    9: (PARAM    , 18001 , _     , 5000  )    ; 4 → param n
   10: (GOSUB    , doble , _     , 1     )
   11: (=        , 1000  , _     , 13002 )
   12: (+        , 13001 , 13002 , 13003 )    ; suma de los dos retornos
   13: (PRINT    , 13003 , _     , _     )

─── QF02 — saluda() como sentencia (función nula) ───
    0: (GOTO     , _     , _     , 3     )
    1: (PRINT_STR, "hola", _     , _     )
    2: (ENDFUNC  , _     , _     , _     )
    3: (ERA      , saluda, _     , _     )
    4: (GOSUB    , saluda, _     , 1     )
    5: (PRINT_STR, "fin" , _     , _     )

─── QF03 — suma(2, 3) usada en asignación ───
    0: (GOTO     , _     , _     , 5     )
    1: (+        , 5000  , 5001  , 13000 )    ; suma: a + b
    2: (=        , 13000 , _     , 5002  )    ; t = a + b
    3: (RETURN   , 5002  , _     , 1001  )
    4: (ENDFUNC  , _     , _     , _     )
    5: (ERA      , suma  , _     , _     )
    6: (PARAM    , 18000 , _     , 5000  )    ; 2 → a
    7: (PARAM    , 18001 , _     , 5001  )    ; 3 → b
    8: (GOSUB    , suma  , _     , 1     )
    9: (=        , 1001  , _     , 13001 )
   10: (=        , 13001 , _     , 1000  )    ; resultado = ...
   11: (PRINT    , 1000  , _     , _     )

─── QF04 — llamada anidada cuadrado(cuadrado(3)) ───
    0: (GOTO     , _     , _     , 4     )
    1: (*        , 5000  , 5000  , 13000 )
    2: (RETURN   , 13000 , _     , 1000  )
    3: (ENDFUNC  , _     , _     , _     )
    4: (ERA      , cuadrado, _   , _     )    ; outer
    5: (ERA      , cuadrado, _   , _     )    ; inner (pila de llamadas)
    6: (PARAM    , 18000 , _     , 5000  )    ; inner: 3 → n
    7: (GOSUB    , cuadrado, _   , 1     )
    8: (=        , 1000  , _     , 13001 )
    9: (PARAM    , 13001 , _     , 5000  )    ; outer: temp → n
   10: (GOSUB    , cuadrado, _   , 1     )
   11: (=        , 1000  , _     , 13002 )
   12: (PRINT    , 13002 , _     , _     )
```

### 4. Directorio de Funciones

```text
[scope global]
Variables globales:
  a                : entero @ 1000
  b                : entero @ 1001

[tabla de constantes]
  3          : entero    @ 18000
  4          : entero    @ 18001

[función 'cuadrado']
Retorna   : entero
Return @  : 1002
Parámetros: (n: entero @ 5000)
Recursos  : locals(int=1, float=0) temps(int=1, float=0) start_quad=1

[función 'potenciaCuatro']
Retorna   : entero
Return @  : 1003
Parámetros: (n: entero @ 5000)
Recursos  : locals(int=2, float=0) temps(int=0, float=0) start_quad=4
Vars locales (excluye parámetros):
  cuad             : entero @ 5001
```

### 5. Cubo semántico

```text
Tabla de resultados de tipo — (izquierda) OP (derecha)
E = entero  F = flotante  - = error de tipos

izquierda \ OP        +   -   *   /   >   <   !=  ==
─────────────────────────────────────────────────────
entero op entero      E   E   E   E   E   E   E   E
entero op flotante    F   F   F   F   E   E   E   E
flotante op entero    F   F   F   F   E   E   E   E
flotante op flotante  F   F   F   F   E   E   E   E

Compatibilidad de asignación (variable : TO = FROM)
TO \ FROM         entero    flotante
entero            OK        Error
flotante          OK        OK
```

## Estructura del código

```
src/
├── lexer.rs          — Tokens y adapter logos → LALRPOP
├── grammar.lalrpop   — Gramática LALR(1) con acciones embebidas (PNs)
├── types.rs          — Type enum + bases de segmentos de memoria
├── semantic_cube.rs  — Op enum, result_type, is_assignable, precedencias
├── func_dir.rs       — VarInfo, FuncInfo, ConstTable, FuncDir + helpers de direcciones
├── quad_gen.rs       — Operand, Quadruple, QuadGen (3 pilas + fila + contadores temps)
├── context.rs        — SemanticContext: wrapper que coordina scope + qg + errores
└── main.rs           — 17 tests de validación + 11 programas de cuádruplos + 4 tests de funciones
```
