# Como correr el programa

1. Instala Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Verifica la instalacion:

```bash
cargo --version
rustc --version
```

3. Abre una terminal.
4. Entra a la carpeta del proyecto:

```bash
cd patito
```

5. Ejecuta el programa:

```bash
cargo run
```

Al final debe aparecer una salida como esta, indicando que todas las pruebas pasaron:

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

Después de los tests, el programa imprime la **fila de cuádruplos** generada para 11 programas de prueba. Cada renglón tiene el formato `índice: (operación, arg1, arg2, resultado)`. Los campos vacíos se imprimen como `_`. Los saltos (`GOTO`, `GOTOF`) usan el campo `resultado` como el índice destino dentro de la fila.

```text
─── Q01 — asignación con expresión mixta (precedencia) ───
  ┌──────────────────────────────────────────────────────┐
  │  FILA DE CUÁDRUPLOS                                  │
  └──────────────────────────────────────────────────────┘
    0: (=        , 2     , _     , a     )
    1: (=        , 3     , _     , b     )
    2: (=        , 4     , _     , c     )
    3: (*        , b     , c     , t1    )
    4: (+        , a     , t1    , t2    )
    5: (=        , t2    , _     , x     )


─── Q02 — paréntesis cambian precedencia ───
  ┌──────────────────────────────────────────────────────┐
  │  FILA DE CUÁDRUPLOS                                  │
  └──────────────────────────────────────────────────────┘
    0: (=        , 2     , _     , a     )
    1: (=        , 3     , _     , b     )
    2: (=        , 4     , _     , c     )
    3: (+        , a     , b     , t1    )
    4: (*        , t1    , c     , t2    )
    5: (=        , t2    , _     , x     )


─── Q03 — área del círculo (mezcla entero/flotante) ───
  ┌──────────────────────────────────────────────────────┐
  │  FILA DE CUÁDRUPLOS                                  │
  └──────────────────────────────────────────────────────┘
    0: (=        , 5     , _     , radio )
    1: (*        , 3.14159, radio , t1    )
    2: (*        , t1    , radio , t2    )
    3: (=        , t2    , _     , area  )
    4: (PRINT_STR, "Radio: ", _     , _     )
    5: (PRINT    , radio , _     , _     )
    6: (PRINT_STR, "Area: ", _     , _     )
    7: (PRINT    , area  , _     , _     )


─── Q04 — si/sino (mayor de dos) ───
  ┌──────────────────────────────────────────────────────┐
  │  FILA DE CUÁDRUPLOS                                  │
  └──────────────────────────────────────────────────────┘
    0: (=        , 15    , _     , x     )
    1: (=        , 8     , _     , y     )
    2: (>        , x     , y     , t1    )
    3: (GOTOF    , t1    , _     , 6     )
    4: (PRINT    , x     , _     , _     )
    5: (GOTO     , _     , _     , 7     )
    6: (PRINT    , y     , _     , _     )


─── Q05 — si sin sino ───
  ┌──────────────────────────────────────────────────────┐
  │  FILA DE CUÁDRUPLOS                                  │
  └──────────────────────────────────────────────────────┘
    0: (=        , 10    , _     , x     )
    1: (>        , x     , 0     , t1    )
    2: (GOTOF    , t1    , _     , 4     )
    3: (PRINT_STR, "positivo", _     , _     )


─── Q06 — while (factorial) ───
  ┌──────────────────────────────────────────────────────┐
  │  FILA DE CUÁDRUPLOS                                  │
  └──────────────────────────────────────────────────────┘
    0: (=        , 5     , _     , n     )
    1: (=        , 1     , _     , fact  )
    2: (=        , 1     , _     , i     )
    3: (<        , i     , n     , t1    )
    4: (GOTOF    , t1    , _     , 10    )
    5: (+        , i     , 1     , t2    )
    6: (=        , t2    , _     , i     )
    7: (*        , fact  , i     , t3    )
    8: (=        , t3    , _     , fact  )
    9: (GOTO     , _     , _     , 3     )
   10: (PRINT_STR, "Factorial: ", _     , _     )
   11: (PRINT    , fact  , _     , _     )


─── Q07 — Fibonacci (while + múltiples asignaciones) ───
  ┌──────────────────────────────────────────────────────┐
  │  FILA DE CUÁDRUPLOS                                  │
  └──────────────────────────────────────────────────────┘
    0: (=        , 10    , _     , n     )
    1: (=        , 0     , _     , a     )
    2: (=        , 1     , _     , b     )
    3: (=        , 0     , _     , i     )
    4: (<        , i     , n     , t1    )
    5: (GOTOF    , t1    , _     , 14    )
    6: (PRINT    , a     , _     , _     )
    7: (+        , a     , b     , t2    )
    8: (=        , t2    , _     , temp  )
    9: (=        , b     , _     , a     )
   10: (=        , temp  , _     , b     )
   11: (+        , i     , 1     , t3    )
   12: (=        , t3    , _     , i     )
   13: (GOTO     , _     , _     , 4     )


─── Q08 — si anidado dentro de while ───
  ┌──────────────────────────────────────────────────────┐
  │  FILA DE CUÁDRUPLOS                                  │
  └──────────────────────────────────────────────────────┘
    0: (=        , 0     , _     , i     )
    1: (=        , 0     , _     , par   )
    2: (<        , i     , 10    , t1    )
    3: (GOTOF    , t1    , _     , 14    )
    4: (<        , i     , 5     , t2    )
    5: (GOTOF    , t2    , _     , 9     )
    6: (+        , par   , i     , t3    )
    7: (=        , t3    , _     , par   )
    8: (GOTO     , _     , _     , 11    )
    9: (-        , par   , i     , t4    )
   10: (=        , t4    , _     , par   )
   11: (+        , i     , 1     , t5    )
   12: (=        , t5    , _     , i     )
   13: (GOTO     , _     , _     , 2     )
   14: (PRINT_STR, "Resultado: ", _     , _     )
   15: (PRINT    , par   , _     , _     )


─── Q09 — relacionales y aritmética combinadas ───
  ┌──────────────────────────────────────────────────────┐
  │  FILA DE CUÁDRUPLOS                                  │
  └──────────────────────────────────────────────────────┘
    0: (=        , 1     , _     , a     )
    1: (=        , 2     , _     , b     )
    2: (=        , 3     , _     , c     )
    3: (*        , b     , c     , t1    )
    4: (+        , a     , t1    , t2    )
    5: (+        , c     , b     , t3    )
    6: (==       , t2    , t3    , t4    )
    7: (=        , t4    , _     , r     )
    8: (PRINT    , r     , _     , _     )


─── Q10 — regresa con cálculo ───
  ┌──────────────────────────────────────────────────────┐
  │  FILA DE CUÁDRUPLOS                                  │
  └──────────────────────────────────────────────────────┘
    0: (*        , n     , n     , t1    )
    1: (RETURN   , t1    , _     , _     )
    2: (PRINT_STR, "cuadrado de 7", _     , _     )


─── Q11 — STRESS TEST: ciclos anidados, ifs encadenados, mezcla de tipos ───
  ┌──────────────────────────────────────────────────────┐
  │  FILA DE CUÁDRUPLOS                                  │
  └──────────────────────────────────────────────────────┘
    0: (=        , 10    , _     , n     )
    1: (=        , 1     , _     , i     )
    2: (=        , 0     , _     , suma  )
    3: (=        , 1     , _     , prod  )
    4: (=        , 0     , _     , max   )
    5: (=        , 100   , _     , min   )
    6: (=        , 0     , _     , contador)
    7: (=        , 1.5   , _     , escala)
    8: (<        , i     , n     , t1    )
    9: (GOTOF    , t1    , _     , 34    )
   10: (+        , suma  , i     , t2    )
   11: (=        , t2    , _     , suma  )
   12: (*        , prod  , i     , t3    )
   13: (=        , t3    , _     , prod  )
   14: (>        , i     , max   , t4    )
   15: (GOTOF    , t4    , _     , 17    )
   16: (=        , i     , _     , max   )
   17: (<        , i     , min   , t5    )
   18: (GOTOF    , t5    , _     , 21    )
   19: (=        , i     , _     , min   )
   20: (GOTO     , _     , _     , 23    )
   21: (+        , contador, 1     , t6    )
   22: (=        , t6    , _     , contador)
   23: (=        , 0     , _     , j     )
   24: (<        , j     , i     , t7    )
   25: (GOTOF    , t7    , _     , 31    )
   26: (+        , suma  , 1     , t8    )
   27: (=        , t8    , _     , suma  )
   28: (+        , j     , 1     , t9    )
   29: (=        , t9    , _     , j     )
   30: (GOTO     , _     , _     , 24    )
   31: (+        , i     , 1     , t10   )
   32: (=        , t10   , _     , i     )
   33: (GOTO     , _     , _     , 8     )
   34: (*        , suma  , escala, t11   )
   35: (=        , t11   , _     , promedio)
   36: (>        , suma  , prod  , t12   )
   37: (GOTOF    , t12   , _     , 41    )
   38: (PRINT_STR, "suma mayor: ", _     , _     )
   39: (PRINT    , suma  , _     , _     )
   40: (GOTO     , _     , _     , 47    )
   41: (==       , suma  , prod  , t13   )
   42: (GOTOF    , t13   , _     , 45    )
   43: (PRINT_STR, "iguales", _     , _     )
   44: (GOTO     , _     , _     , 47    )
   45: (PRINT_STR, "prod mayor: ", _     , _     )
   46: (PRINT    , prod  , _     , _     )
   47: (PRINT_STR, "max: ", _     , _     )
   48: (PRINT    , max   , _     , _     )
   49: (PRINT_STR, "min: ", _     , _     )
   50: (PRINT    , min   , _     , _     )
   51: (PRINT_STR, "contador: ", _     , _     )
   52: (PRINT    , contador, _     , _     )
   53: (PRINT_STR, "promedio: ", _     , _     )
   54: (PRINT    , promedio, _     , _     )
```

Si sale "command not found: cargo", ejecuta esto y vuelve a correr cargo run:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```