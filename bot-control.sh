#!/bin/bash

# Polymarket Arbitrage Bot Control Script
# Usage: ./bot-control.sh [start|stop|restart|logs|status]

set -e

COMMAND=${1:-status}
FILTER=${2:-}

case "$COMMAND" in
    start)
        echo "🚀 Starting Polymarket Bot..."
        docker-compose up -d
        sleep 5
        docker-compose ps
        echo "✅ Bot started. View logs with: ./bot-control.sh logs"
        ;;

    stop)
        echo "⛔ Stopping Polymarket Bot..."
        docker-compose down
        echo "✅ Bot stopped"
        ;;

    restart)
        echo "🔄 Restarting Polymarket Bot..."
        docker-compose restart bot
        sleep 5
        echo "✅ Bot restarted. View logs with: ./bot-control.sh logs"
        ;;

    logs)
        echo "📊 Live logs (Ctrl+C to exit)..."
        if [ -z "$FILTER" ]; then
            # Default: show market data, arbitrage detections, errors
            docker-compose logs -f bot 2>&1 | grep -v "Fetching binary"
        else
            # Custom filter
            docker-compose logs -f bot 2>&1 | grep "$FILTER"
        fi
        ;;

    logs-all)
        echo "📊 All logs including API calls (Ctrl+C to exit)..."
        docker-compose logs -f bot
        ;;

    logs-arb)
        echo "🎯 Arbitrage opportunities only (Ctrl+C to exit)..."
        docker-compose logs -f bot 2>&1 | grep -i "arbitrage\|opportunity\|combined"
        ;;

    logs-low)
        echo "💰 Markets with combined price < $0.90 (Ctrl+C to exit)..."
        docker-compose logs -f bot 2>&1 | grep "price.*<.*0.9"
        ;;

    status)
        echo "📈 Current Bot Status:"
        docker-compose ps
        echo ""
        echo "Recent activity (last 30 lines):"
        docker-compose logs --tail 30 bot 2>&1 | grep -v "Fetching binary" || echo "No activity yet"
        ;;

    stats)
        echo "📊 Statistics from logs..."
        TOTAL=$(docker-compose logs bot 2>&1 | grep -c "Fetched 100" || echo 0)
        echo "✅ API calls: $TOTAL cycles"
        DB_SIZE=$(docker exec polymarket_postgres psql -U polymarket -d polymarket_bot -t -c "SELECT COUNT(*) FROM markets;" 2>/dev/null || echo "0")
        echo "📦 Markets in DB: $DB_SIZE"
        ;;

    clean)
        echo "🗑️  Cleaning up Docker resources..."
        docker-compose down -v
        echo "✅ Cleaned"
        ;;

    *)
        echo "Usage: $0 [command] [filter]"
        echo ""
        echo "Commands:"
        echo "  start          - Start the bot"
        echo "  stop           - Stop the bot"
        echo "  restart        - Restart the bot"
        echo "  status         - Show bot status and recent activity"
        echo "  logs           - Show live logs (excluding API fetches)"
        echo "  logs-all       - Show all logs including API calls"
        echo "  logs-arb       - Show only arbitrage detections"
        echo "  logs-low       - Show only markets with price < \$0.90"
        echo "  stats          - Show statistics"
        echo "  clean          - Stop and remove containers/volumes"
        echo ""
        echo "Examples:"
        echo "  ./bot-control.sh start"
        echo "  ./bot-control.sh logs"
        echo "  ./bot-control.sh logs 'ERROR'"
        echo "  ./bot-control.sh logs-low"
        ;;
esac
