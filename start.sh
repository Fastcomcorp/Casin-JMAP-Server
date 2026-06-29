#!/bin/bash

# Casin JMAP Server - Local Startup Script
# Copyright (c) 2026 Fastcomcorp LLC

echo "=========================================="
echo "    Starting Casin JMAP Server Stack      "
echo "=========================================="

# 1. Start the Rust JMAP API
echo "[1/3] Starting Rust JMAP Server (Axum) on port 3000..."
cd rust-services/jmap-scheduler
export JMAP_DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/casin_jmap"
# Run in background
cargo run --bin jmap-scheduler > rust_server.log 2>&1 &
RUST_PID=$!
cd ../../

# 2. Start the Elixir Realtime Cluster
echo "[2/3] Starting Elixir Realtime WebSocket & Sync Cluster..."
cd elixir_realtime
export DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/casin_jmap"
# Run mix setup to ensure dependencies and DB migrations run
mix deps.get > /dev/null 2>&1
# Run in background
mix run --no-halt > elixir_server.log 2>&1 &
ELIXIR_PID=$!
cd ..

echo "[3/3] Stack is LIVE!"
echo "------------------------------------------"
echo "-> JMAP API Gateway: http://127.0.0.1:3000/jmap"
echo "-> Elixir WebSockets: Connected via libcluster"
echo "-> NATS Firehose: Port 4222"
echo "------------------------------------------"
echo "Press Ctrl+C to shut down all services."

trap "echo 'Shutting down Casin Server...'; kill $RUST_PID $ELIXIR_PID; exit" SIGINT SIGTERM

# Keep script running
wait $RUST_PID $ELIXIR_PID
