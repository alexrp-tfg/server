#!/bin/bash
set -e

echo "Running database migrations..."
diesel migration run

echo "Creating admin user..."
./server create-admin || echo "Admin user creation failed or already exists"

echo "Starting server..."
exec ./server
