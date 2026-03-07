#!/bin/bash

# Polymarket Bot Quick Start Script

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo "======================================"
echo "Polymarket Arbitrage Bot - Quick Start"
echo "======================================"
echo ""

# Check if .env exists
if [ ! -f "$SCRIPT_DIR/.env" ]; then
    echo "❌ .env file not found!"
    echo "Creating .env from .env.example..."
    cp "$SCRIPT_DIR/.env.example" "$SCRIPT_DIR/.env"
    echo ""
    echo "⚠️  IMPORTANT: Edit .env and set your POLY_PRIVATE_KEY before starting!"
    echo "   nano .env"
    exit 1
fi

# Check if POLY_PRIVATE_KEY is set
if ! grep -q "POLY_PRIVATE_KEY=0x" "$SCRIPT_DIR/.env"; then
    echo "❌ POLY_PRIVATE_KEY not configured in .env"
    echo "Edit your .env file:"
    echo "   nano .env"
    exit 1
fi

echo "✅ Configuration found"
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not installed"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose not installed"
    exit 1
fi

echo "✅ Docker & Docker Compose available"
echo ""

# Start services
echo "🚀 Starting Polymarket Bot..."
echo ""

docker-compose up -d

echo ""
echo "✅ Services started!"
echo ""
echo "📊 Monitoring..."
echo "   docker-compose logs -f bot"
echo ""
echo "📈 View trade history:"
echo "   docker-compose exec postgres psql -U polymarket_user -d polymarket_bot"
echo "   SELECT * FROM orders ORDER BY created_at DESC LIMIT 10;"
echo ""
echo "🛑 Stop bot:"
echo "   docker-compose down"
echo ""
echo "For detailed setup, see SETUP.md"
