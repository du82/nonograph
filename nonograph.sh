#!/bin/bash

# Nonograph Management Script
# This script helps manage the Nonograph service and Tor hidden service

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

show_header() {
    echo "╔══════════════════════════════════════╗"
    echo "║         Nonograph Manager            ║"
    echo "║    Anonymous Publishing Service      ║"
    echo "╚══════════════════════════════════════╝"
    echo
}

show_help() {
    show_header
    echo "Usage: $0 [command]"
    echo
    echo "Commands:"
    echo "  start       Start Nonograph service"
    echo "  stop        Stop Nonograph service"
    echo "  restart     Restart Nonograph service"
    echo "  status      Show service status"
    echo "  logs        Show recent logs"
    echo "  onion       Show onion address"
    echo "  test        Create a test post"
    echo "  build       Build the project"
    echo "  clean       Clean build artifacts"
    echo "  tor-start   Start Tor service"
    echo "  tor-stop    Stop Tor service"
    echo "  tor-restart Restart Tor service"
    echo "  help        Show this help message"
    echo
}

start_service() {
    echo "🚀 Starting Nonograph..."

    # Check if already running
    if pgrep -f "nonograph" > /dev/null; then
        echo "⚠️  Nonograph is already running"
        return 1
    fi

    # Start in background
    nohup cargo run --release > nonograph.log 2>&1 &

    # Wait a moment and check if it started
    sleep 3
    if pgrep -f "nonograph" > /dev/null; then
        echo "✅ Nonograph started successfully"
        echo "🌐 Local access: http://localhost:8009"
    else
        echo "❌ Failed to start Nonograph"
        echo "📝 Check logs: tail -f nonograph.log"
        return 1
    fi
}

stop_service() {
    echo "🛑 Stopping Nonograph..."

    if ! pgrep -f "nonograph" > /dev/null; then
        echo "⚠️  Nonograph is not running"
        return 1
    fi

    pkill -f "nonograph"
    sleep 2

    if ! pgrep -f "nonograph" > /dev/null; then
        echo "✅ Nonograph stopped successfully"
    else
        echo "⚠️  Force killing Nonograph..."
        pkill -9 -f "nonograph"
        sleep 1
        if ! pgrep -f "nonograph" > /dev/null; then
            echo "✅ Nonograph force stopped"
        else
            echo "❌ Failed to stop Nonograph"
            return 1
        fi
    fi
}

restart_service() {
    echo "🔄 Restarting Nonograph..."
    stop_service
    sleep 1
    start_service
}

show_status() {
    if [ -f "status.sh" ]; then
        ./status.sh
    else
        show_header

        # Check Nonograph status
        NONOGRAPH_PID=$(pgrep -f "nonograph" | head -1)
        if [ -n "$NONOGRAPH_PID" ]; then
            echo "✅ Nonograph Status: RUNNING (PID: $NONOGRAPH_PID)"
        else
            echo "❌ Nonograph Status: NOT RUNNING"
        fi

        # Check Tor status
        TOR_STATUS=$(systemctl is-active tor 2>/dev/null)
        if [ "$TOR_STATUS" = "active" ]; then
            echo "✅ Tor Service: RUNNING"
        else
            echo "❌ Tor Service: NOT RUNNING"
        fi

        echo
    fi
}

show_logs() {
    echo "📝 Recent Nonograph logs:"
    echo "────────────────────────────────────"
    if [ -f "nonograph.log" ]; then
        tail -20 nonograph.log
    else
        echo "No log file found"
    fi
    echo "────────────────────────────────────"
    echo "💡 Use 'tail -f nonograph.log' to follow logs in real-time"
}

show_onion() {
    echo "🧅 Tor Hidden Service Information:"
    echo "──────────────────────────────────"

    if sudo test -f "/var/lib/tor/nonograph/hostname" 2>/dev/null; then
        ONION_ADDRESS=$(sudo cat /var/lib/tor/nonograph/hostname 2>/dev/null)
        if [ -n "$ONION_ADDRESS" ]; then
            echo "Onion Address: http://$ONION_ADDRESS"
            echo
            echo "📋 Copy to clipboard:"
            echo "echo 'http://$ONION_ADDRESS' | xclip -selection clipboard"
            echo
            echo "🌐 Access via Tor Browser:"
            echo "Open Tor Browser and navigate to: http://$ONION_ADDRESS"
        else
            echo "❌ Could not read onion address"
        fi
    else
        echo "❌ Hidden service not configured or not accessible"
        echo "💡 Make sure Tor is running and the hidden service is set up"
    fi
}

create_test() {
    echo "🧪 Creating test post..."

    # Check if service is running
    if ! curl -s --max-time 5 http://localhost:8009 > /dev/null 2>&1; then
        echo "❌ Nonograph is not accessible"
        echo "💡 Make sure the service is running with: $0 start"
        return 1
    fi

    # Create test post
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
    curl -s -X POST http://localhost:8009/create \
        -d "title=Test Post - $TIMESTAMP" \
        -d "content=This is a test post created at $TIMESTAMP to verify Nonograph is working correctly as a Tor hidden service." \
        -H "Content-Type: application/x-www-form-urlencoded" > /dev/null

    if [ $? -eq 0 ]; then
        echo "✅ Test post created successfully"
        echo "🌐 Check at: http://localhost:8009"
    else
        echo "❌ Failed to create test post"
        return 1
    fi
}

build_project() {
    echo "🔨 Building Nonograph..."
    cargo build --release
    if [ $? -eq 0 ]; then
        echo "✅ Build completed successfully"
    else
        echo "❌ Build failed"
        return 1
    fi
}

clean_project() {
    echo "🧹 Cleaning build artifacts..."
    cargo clean
    if [ $? -eq 0 ]; then
        echo "✅ Clean completed successfully"
    else
        echo "❌ Clean failed"
        return 1
    fi
}

start_tor() {
    echo "🧅 Starting Tor service..."
    sudo systemctl start tor
    if [ $? -eq 0 ]; then
        echo "✅ Tor service started"
    else
        echo "❌ Failed to start Tor service"
        return 1
    fi
}

stop_tor() {
    echo "🛑 Stopping Tor service..."
    sudo systemctl stop tor
    if [ $? -eq 0 ]; then
        echo "✅ Tor service stopped"
    else
        echo "❌ Failed to stop Tor service"
        return 1
    fi
}

restart_tor() {
    echo "🔄 Restarting Tor service..."
    sudo systemctl restart tor
    if [ $? -eq 0 ]; then
        echo "✅ Tor service restarted"
    else
        echo "❌ Failed to restart Tor service"
        return 1
    fi
}

# Main command handler
case "${1:-help}" in
    "start")
        start_service
        ;;
    "stop")
        stop_service
        ;;
    "restart")
        restart_service
        ;;
    "status")
        show_status
        ;;
    "logs")
        show_logs
        ;;
    "onion")
        show_onion
        ;;
    "test")
        create_test
        ;;
    "build")
        build_project
        ;;
    "clean")
        clean_project
        ;;
    "tor-start")
        start_tor
        ;;
    "tor-stop")
        stop_tor
        ;;
    "tor-restart")
        restart_tor
        ;;
    "help"|*)
        show_help
        ;;
esac
