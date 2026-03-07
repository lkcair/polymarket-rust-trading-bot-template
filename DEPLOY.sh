#!/bin/bash

# Polymarket Bot - Deployment Script
# This script guides you through setting up and deploying the bot

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "🚀 Polymarket Bot Deployment"
echo "=========================================="
echo ""

# Step 1: Check if .env exists
if [ ! -f "$SCRIPT_DIR/.env" ]; then
    echo "📝 Step 1: Creating .env configuration file..."
    cp "$SCRIPT_DIR/.env.example" "$SCRIPT_DIR/.env"
    echo "✅ Created .env from .env.example"
    echo ""
    echo "⚠️  IMPORTANT: You must edit .env with your MetaMask private key!"
    echo ""
    echo "To get your private key:"
    echo "  1. Open MetaMask"
    echo "  2. Click ⋯ (three dots) → Account Details"
    echo "  3. Click 'Export Private Key'"
    echo "  4. Copy the key (starts with 0x)"
    echo ""
    echo "Then edit .env:"
    echo "  nano .env"
    echo "  # Set POLY_PRIVATE_KEY=0x[your-key-here]"
    echo ""
    exit 1
fi

echo "✅ .env file found"
echo ""

# Step 2: Verify private key is configured
if ! grep -q "POLY_PRIVATE_KEY=" "$SCRIPT_DIR/.env"; then
    echo "❌ POLY_PRIVATE_KEY not configured in .env"
    echo ""
    echo "Please edit .env and set your Polygon private key:"
    echo "  POLY_PRIVATE_KEY=[your-hex-key-here]"
    echo ""
    echo "Then run this script again."
    exit 1
fi

# Check if private key is actually set (not empty)
PRIVATE_KEY=$(grep "^POLY_PRIVATE_KEY=" "$SCRIPT_DIR/.env" | cut -d'=' -f2)
if [ -z "$PRIVATE_KEY" ]; then
    echo "❌ POLY_PRIVATE_KEY is empty in .env"
    exit 1
fi

echo "📝 Step 2: Validating private key format..."
KEY=$(echo "$PRIVATE_KEY" | tr -d ' ')

# Accept both formats: plain hex (62-65 chars) or 0x prefixed (66 chars)
if [[ "$KEY" =~ ^0x[0-9a-fA-F]{64}$ ]] || [[ "$KEY" =~ ^[0-9a-fA-F]{62,65}$ ]]; then
    echo "✅ Private key format valid"
else
    echo "⚠️  Warning: Private key format may be invalid"
    echo "   Expected: plain hex (62-65 chars) or 0x prefixed (66 chars)"
    echo "   Current length: ${#KEY} chars"
    echo ""
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi
echo "✅ Private key format validated"
echo ""

# Step 3: Check Docker
echo "📝 Step 3: Checking Docker installation..."
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed"
    echo "   Visit: https://docs.docker.com/get-docker/"
    exit 1
fi
echo "✅ Docker available: $(docker --version)"

if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed"
    echo "   Visit: https://docs.docker.com/compose/install/"
    exit 1
fi
echo "✅ Docker Compose available"
echo ""

# Step 4: Build and start containers
echo "📝 Step 4: Building and starting services..."
echo "   (This may take 2-3 minutes on first run)"
echo ""

docker-compose up -d
echo "✅ Services started!"
echo ""

# Wait for postgres to be healthy
echo "📝 Step 5: Waiting for database to initialize..."
RETRY_COUNT=0
MAX_RETRIES=30
while ! docker-compose exec -T postgres pg_isready -U polymarket_user > /dev/null 2>&1; do
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -gt $MAX_RETRIES ]; then
        echo "❌ PostgreSQL failed to start"
        echo "   Logs:"
        docker-compose logs postgres
        exit 1
    fi
    echo -n "."
    sleep 1
done
echo ""
echo "✅ Database is ready!"
echo ""

# Step 6: Verify schema
echo "📝 Step 6: Verifying database schema..."
TABLES=$(docker-compose exec -T postgres psql -U polymarket_user -d polymarket_bot -c "SELECT COUNT(*) FROM pg_tables WHERE schemaname='public';" 2>/dev/null || echo "0")
if [ "$TABLES" -gt 0 ]; then
    echo "✅ Database schema initialized ($TABLES tables)"
else
    echo "⚠️  Warning: Database schema may not have initialized"
fi
echo ""

# Step 7: Display status
echo "=========================================="
echo "✅ DEPLOYMENT COMPLETE!"
echo "=========================================="
echo ""
echo "📊 Bot Status:"
docker-compose ps
echo ""

echo "📋 Paper Trading Mode: ENABLED"
echo "   The bot will simulate trades without real money"
echo ""

echo "📝 Next Steps:"
echo ""
echo "1. Monitor logs (watch in real-time):"
echo "   docker-compose logs -f bot"
echo ""

echo "2. Check for arbitrage opportunities:"
echo "   docker-compose logs -f bot | grep -i arbitrage"
echo ""

echo "3. View trade history:"
echo "   docker-compose exec postgres psql -U polymarket_user -d polymarket_bot"
echo "   SELECT * FROM opportunities ORDER BY timestamp DESC LIMIT 5;"
echo "   \\q  (to exit)"
echo ""

echo "4. When ready to test live trading:"
echo "   a. Edit .env: PAPER_TRADING=false"
echo "   b. Restart: docker-compose restart bot"
echo "   c. Monitor closely: docker-compose logs -f bot"
echo ""

echo "⚠️  IMPORTANT REMINDERS:"
echo "   • Start with PAPER_TRADING=true (default)"
echo "   • Validate arbitrage detection for 24+ hours"
echo "   • Only enable live trading after validation"
echo "   • Never commit .env to git (already protected)"
echo ""

echo "📚 For detailed setup instructions, see:"
echo "   DEPLOYMENT_GUIDE.md - Complete deployment walkthrough"
echo "   SETUP.md - Additional configuration & troubleshooting"
echo "   README.md - Architecture and strategy overview"
echo ""
