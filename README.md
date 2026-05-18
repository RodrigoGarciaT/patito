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
═══════════════════════════════════════════════════════
  Patito — pruebas léxico/sintácticas
═══════════════════════════════════════════════════════

  [OK]   Ejemplo 1 — programa mínimo
  [OK]   Ejemplo 2 — vars, asignación, expresiones flotantes
  [OK]   Ejemplo 3 — condicional si/sino
  [OK]   Ejemplo 4 — ciclo mientras/haz
  [OK]   Ejemplo 5 — funciones con regresa y llamadas anidadas
  [OK]   Ejemplo 6 — Fibonacci (múltiples vars en una declaración)

  — Errores de sintaxis —
  [OK]   Error esperado: falta id de programa
  [OK]   Error esperado: falta ; en asignación
  [OK]   Error esperado: falta ; al final del si

  — Errores semánticos —
  [OK]   SemError 1 — variable duplicada  →  PN2: declarar variable: VariableDuplicada("x")
  [OK]   SemError 2 — asignacion a variable no declarada  →  PN11: validar asignación: VariableNoDeclarada("y")
  [OK]   SemError 3 — asignacion flotante a entero  →  PN11: validar asignación: TiposIncompatibles { izq: Entero, der: Flotante, op: Asig }
  [OK]   SemError 4 — funcion duplicada  →  PN3: declarar función: FuncionDuplicada("foo")
  [OK]   SemError 5 — llamada a funcion no declarada  →  PN7: buscar función: FuncionNoDeclarada("foo")
  [OK]   SemError 6 — regresa fuera de funcion (en global)  →  PN12: validar regresa: RegresaFueraDeFuncion
  [OK]   SemError 7 — regresa con tipo incompatible  →  PN12: validar regresa: RetornoIncompatible { esperado: Entero, obtenido: Flotante }
  [OK]   SemError 8 — regresa en funcion con retorno nula  →  PN12: validar regresa: RetornoEnFuncionNula

═══════════════════════════════════════════════════════
  Resultado: 17/17 pruebas correctas
═══════════════════════════════════════════════════════
```

Si sale "command not found: cargo", ejecuta esto y vuelve a correr cargo run:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```
