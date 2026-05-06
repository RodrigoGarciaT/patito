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

	— Casos que deben reportar error —
	[OK]   Error esperado: falta id de programa
	[OK]   Error esperado: falta ; en asignación
	[OK]   Error esperado: falta ; al final del si

═══════════════════════════════════════════════════════
	Resultado: 9/9 pruebas correctas
```

Si sale "command not found: cargo", ejecuta esto y vuelve a correr cargo run:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```
