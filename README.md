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

Formato: `índice: (operación, arg1, arg2, resultado)`. Los campos vacíos se imprimen como `_`. Los argumentos son direcciones numéricas: `1000-1999` son globales enteras, `5000-5999` locales enteras, `13000-13999` temporales enteras, `18000-18999` constantes enteras (similar para flotantes con base +1000).

**GOTO inicial:** el cuádruplo `0` de todo programa es un `GOTO _ _ N` donde `N` es el índice del primer cuádruplo del bloque `inicio`. Cuando hay funciones declaradas, esos cuerpos van en los cuádruplos `1..N-1`, y el GOTO se encarga de brincarlos para que la ejecución arranque en `inicio` sin caer por accidente dentro de una función. Cuando no hay funciones, el GOTO simplemente salta a `1` (el siguiente cuádruplo).

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
    0: (GOTO     , _     , _     , 4     )
    1: (+        , 5000  , 5000  , 13000 )
    2: (RETURN   , 13000 , _     , 1000  )
    3: (ENDFUNC  , _     , _     , _     )
    4: (ERA      , doble , _     , _     )
    5: (PARAM    , 18000 , _     , 5000  )
    6: (GOSUB    , doble , _     , 1     )
    7: (=        , 1000  , _     , 13001 )
    8: (ERA      , doble , _     , _     )
    9: (PARAM    , 18001 , _     , 5000  )
   10: (GOSUB    , doble , _     , 1     )
   11: (=        , 1000  , _     , 13002 )
   12: (+        , 13001 , 13002 , 13003 )
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
    1: (+        , 5000  , 5001  , 13000 )
    2: (=        , 13000 , _     , 5002  )
    3: (RETURN   , 5002  , _     , 1001  )
    4: (ENDFUNC  , _     , _     , _     )
    5: (ERA      , suma  , _     , _     )
    6: (PARAM    , 18000 , _     , 5000  )
    7: (PARAM    , 18001 , _     , 5001  )
    8: (GOSUB    , suma  , _     , 1     )
    9: (=        , 1001  , _     , 13001 )
   10: (=        , 13001 , _     , 1000  )
   11: (PRINT    , 1000  , _     , _     )

─── QF04 — llamada anidada cuadrado(cuadrado(3)) ───
    0: (GOTO     , _     , _     , 4     )
    1: (*        , 5000  , 5000  , 13000 )
    2: (RETURN   , 13000 , _     , 1000  )
    3: (ENDFUNC  , _     , _     , _     )
    4: (ERA      , cuadrado, _   , _     )
    5: (ERA      , cuadrado, _   , _     )
    6: (PARAM    , 18000 , _     , 5000  )
    7: (GOSUB    , cuadrado, _   , 1     )
    8: (=        , 1000  , _     , 13001 )
    9: (PARAM    , 13001 , _     , 5000  )
   10: (GOSUB    , cuadrado, _   , 1     )
   11: (=        , 1000  , _     , 13002 )
   12: (PRINT    , 13002 , _     , _     )
```

En QF01-QF04 el cuádruplo `0` siempre brinca al primer cuádruplo de `inicio`: en QF01 va a `4` (porque el cuerpo de `doble` ocupa los quads `1-3`), en QF02 va a `3` (cuerpo de `saluda` en `1-2`), etc.

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
patito/
├── Cargo.toml             — dependencias (lalrpop, logos, serde, serde_json)
├── build.rs               — script de compilación de LALRPOP
├── run.sh                 — script bash que compila Patito → JSON → ejecuta en VM
├── vm.cpp                 — Máquina virtual en C++ que ejecuta los .obj
├── src/
│   ├── lexer.rs           — Tokens y adapter logos → LALRPOP
│   ├── grammar.lalrpop    — Gramática LALR(1) con acciones embebidas (PNs)
│   ├── types.rs           — Type enum + bases de segmentos de memoria
│   ├── semantic_cube.rs   — Op enum, result_type, is_assignable, precedencias
│   ├── func_dir.rs        — VarInfo, FuncInfo, ConstTable, FuncDir + helpers de direcciones
│   ├── quad_gen.rs        — Operand, Quadruple, QuadGen (3 pilas + fila + contadores temps)
│   ├── context.rs         — SemanticContext: wrapper que coordina scope + qg + errores
│   ├── vm_payload.rs      — Capa de exportación: convierte SemanticContext a JSON
│   └── main.rs            — Test runner + modo "compilar archivo .patito → .obj"
└── examples/              — Programas de prueba en Patito
    ├── factorial_main.patito
    ├── factorial_func.patito
    ├── fib_main.patito
    ├── fib_recursivo.patito
    ├── mul_div.patito
    ├── mcd.patito
    └── suma_cuadrados.patito
```

## Entrega 5 — Máquina Virtual y pipeline completo

La VM ejecuta los cuádruplos generados por el compilador. El puente es un archivo `.obj` en JSON que contiene todo lo que la VM necesita: cuádruplos, tabla de constantes, tamaños de globales, metadata por función y metadata de main.

### Pipeline

```
programa.patito  ──cargo run──▶  programa.obj  ──./vm──▶  output
   (texto)          (compilador      (JSON)        (VM)        (terminal)
                     en Rust)                   en C++
```

### Setup (una sola vez)

```bash
brew install nlohmann-json                # librería JSON para C++
cd patito                                  # carpeta del proyecto
g++ -std=c++17 -O2 -I/opt/homebrew/include -o vm vm.cpp   # compila la VM
```

### Correr un programa

Hay un script `run.sh` que orquesta todo (compila VM + compila .patito + ejecuta). Le pasas la ruta del programa **sin extensión**:

```bash
./run.sh examples/factorial_main
```

Equivalente manual:

```bash
cargo run --quiet -- examples/factorial_main.patito   # genera examples/factorial_main.obj
./vm examples/factorial_main.obj                       # ejecuta
```

### Salida esperada de cada ejemplo

#### 1. `factorial_main` — Factorial calculado en el main con un `mientras`

```text
Factorial de 5:
120
```

#### 2. `factorial_func` — Factorial como función con parámetro entero y `regresa`

```text
5! =
120
6! =
720
7! =
5040
```

#### 3. `fib_main` — Fibonacci iterativo en el main (primeros 10 términos)

```text
Primeros 10 Fibonacci:
0
1
1
2
3
5
8
13
21
34
```

#### 4. `fib_recursivo` — Fibonacci como función **recursiva** (primeros 20 términos)

Cada `fib(i)` se expande en llamadas anidadas; `fib(19)` solo hace ~13,500 llamadas recursivas. Stress test del `call_stack` y del back-patching de funciones.

```text
0
1
1
2
3
5
8
13
21
34
55
89
144
233
377
610
987
1597
2584
4181
```

#### 5. `mul_div` — Aritmética mixta con promoción int↔float

```text
120          ← 20 * 6 (int × int → int)
3            ← 20 / 6 (int / int → int truncada)
10           ← 5.0 * 2.0 (float × float)
2.5          ← 5.0 / 2.0
40           ← 20 * 2.0 (int × float → float, promueve el 20)
0.833333     ← 5.0 / 6 (float / int → float)
```

#### 6. `mcd` — Máximo Común Divisor (algoritmo de Euclides). Sin operador módulo nativo, lo simula con `a - (a/b)*b`. Demuestra reasignación de parámetros, ciclo con `!=`, paréntesis en expresiones.

```text
MCD(48, 18) =
6
MCD(100, 75) =
25
MCD(17, 5) =
1
MCD(81, 27) =
27
```

#### 7. `suma_cuadrados` — Función llamando a otra función. `sumaCuadrados(n)` invoca a `cuadrado(i)` dentro de un ciclo, demostrando manejo correcto del `call_stack` y del activation record pendiente.

```text
Suma de cuadrados de 1 a 3:
14
Suma de cuadrados de 1 a 5:
55
Suma de cuadrados de 1 a 10:
385
```

### Features cubiertas por los ejemplos

| Característica | Cubierta por |
|---|---|
| Asignación de variables | Todos |
| Expresiones aritméticas con precedencia | Todos |
| `escribe()` con letrero | Casi todos |
| `escribe()` con expresión | Todos |
| Ciclo `mientras / haz` | factorial_main, factorial_func, fib_main, fib_recursivo, mcd, suma_cuadrados |
| Decisión `si / sino` | fib_recursivo |
| Declaración de función con `regresa` | factorial_func, fib_recursivo, mcd, suma_cuadrados |
| Parámetros con tipo | factorial_func, fib_recursivo, mcd, suma_cuadrados |
| Variables locales | factorial_func, mcd, suma_cuadrados |
| Llamada a función desde main | factorial_func, fib_recursivo, mcd, suma_cuadrados |
| Llamada a función desde otra función | suma_cuadrados |
| Llamadas **recursivas** | fib_recursivo |
| Llamadas anidadas en expresión | fib_recursivo (`fib(n-1) + fib(n-2)`) |
| Promoción de tipos en aritmética y asignación | mul_div |
| División entera vs flotante | mul_div, mcd |
| Comparación con `!=` | mcd |
