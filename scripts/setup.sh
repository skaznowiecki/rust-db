#!/bin/bash

DB=ecommerce

cargo run -- exec "CREATE DATABASE $DB"

cargo run -- exec --db $DB "CREATE TABLE usuarios (id SERIAL PRIMARY KEY, email VARCHAR(255) NOT NULL UNIQUE, nombre VARCHAR(100) NOT NULL, activo BOOLEAN DEFAULT true)"

cargo run -- exec --db $DB "CREATE TABLE productos (id SERIAL PRIMARY KEY, nombre VARCHAR(200) NOT NULL, precio INTEGER NOT NULL, stock INTEGER DEFAULT 0)"

cargo run -- exec --db $DB "CREATE TABLE ordenes (id SERIAL PRIMARY KEY, usuario_id INTEGER NOT NULL, total INTEGER NOT NULL, estado VARCHAR(50) DEFAULT 'pendiente')"

cargo run -- exec --db $DB "CREATE TABLE orden_items (id SERIAL PRIMARY KEY, orden_id INTEGER NOT NULL, producto_id INTEGER NOT NULL, cantidad INTEGER NOT NULL)"

cargo run -- exec --db $DB "CREATE TABLE categorias (id SERIAL PRIMARY KEY, nombre VARCHAR(100) NOT NULL UNIQUE, descripcion TEXT)"

cargo run -- exec "DROP DATABASE $DB"
