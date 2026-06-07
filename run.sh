#!/bin/bash
# Uso: ./run.sh examples/fib_recursivo
#      (sin la extensión .patito al final)
#
# Hace 3 cosas:
#   1. Recompila la VM de C++ (rápido si nada cambió)
#   2. Compila el .patito → .obj
#   3. Ejecuta el .obj

set -e   # Si algún comando falla, detente

if [ -z "$1" ]; then
    echo "Uso: $0 <ruta-sin-extensión>"
    echo "Ejemplo: $0 examples/fib_recursivo"
    exit 1
fi

PROGRAMA="$1"

echo "──────────────────────────────────────────────"
echo "  Compilando VM (C++)..."
echo "──────────────────────────────────────────────"
g++ -std=c++17 -O2 -I/opt/homebrew/include -o vm vm.cpp

echo "──────────────────────────────────────────────"
echo "  Compilando ${PROGRAMA}.patito → ${PROGRAMA}.obj"
echo "──────────────────────────────────────────────"
cargo run --quiet -- "${PROGRAMA}.patito"

echo "──────────────────────────────────────────────"
echo "  Ejecutando ${PROGRAMA}.obj"
echo "──────────────────────────────────────────────"
./vm "${PROGRAMA}.obj"
