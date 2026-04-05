#!/bin/bash

DB=ecommerce
BIN=./target/release/db
ROWS=100000

cargo build --release 2>&1

# Start server
$BIN start

CATEGORIAS=("Electrónica" "Ropa" "Hogar" "Deportes" "Libros" "Juguetes" "Alimentos" "Herramientas" "Salud" "Automotor")
MARCAS=("Samsung" "Nike" "Sony" "Apple" "Adidas" "LG" "Philips" "Bosch" "HP" "Dell" "Lenovo" "Xiaomi" "Bose" "Puma" "Reebok")
ESTADOS=("activo" "inactivo" "agotado" "descontinuado")

# Generate SQL file
echo "Generating $ROWS inserts..."
{
    echo "DROP DATABASE $DB;"
    echo "CREATE DATABASE $DB;"
    echo "USE $DB;"
    echo "CREATE TABLE productos (id SERIAL PRIMARY KEY, nombre VARCHAR(200) NOT NULL, descripcion TEXT, precio INTEGER NOT NULL, stock INTEGER DEFAULT 0, categoria VARCHAR(100) NOT NULL, marca VARCHAR(100), sku VARCHAR(50) NOT NULL, peso INTEGER, estado VARCHAR(50) DEFAULT 'activo');"
    for i in $(seq 1 $ROWS); do
        CAT=${CATEGORIAS[$((RANDOM % ${#CATEGORIAS[@]}))]}
        MARCA=${MARCAS[$((RANDOM % ${#MARCAS[@]}))]}
        ESTADO=${ESTADOS[$((RANDOM % ${#ESTADOS[@]}))]}
        PRECIO=$((RANDOM % 50000 + 100))
        STOCK=$((RANDOM % 1000))
        PESO=$((RANDOM % 5000 + 50))
        SKU="SKU-$(printf '%07d' $i)"
        echo "INSERT INTO productos (nombre, descripcion, precio, stock, categoria, marca, sku, peso, estado) VALUES ('$MARCA Producto $i', 'Descripcion detallada del producto numero $i de la categoria $CAT fabricado por $MARCA', $PRECIO, $STOCK, '$CAT', '$MARCA', '$SKU', $PESO, '$ESTADO');"
    done
} > /tmp/bench_setup.sql

# Run via REPL connected to server
echo "Inserting $ROWS rows..."
START=$(date +%s%N)

$BIN connect < /tmp/bench_setup.sql > /dev/null 2>&1

END=$(date +%s%N)
ELAPSED=$(( (END - START) / 1000000 ))

echo "---"
echo "Inserted $ROWS rows in ${ELAPSED}ms ($(echo "scale=2; $ELAPSED / 1000" | bc)s)"
echo "Avg per insert: $(echo "scale=3; $ELAPSED / $ROWS" | bc)ms"

LINES=$(wc -l < "./data/$DB/1001/data")
echo "Rows in data file: $LINES"

# File size
SIZE=$(du -h "./data/$DB/1001/data" | cut -f1)
echo "Data file size: $SIZE"

# Stop server
$BIN stop
