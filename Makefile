.PHONY: help build up down logs clean dev dev-down dev-logs test restart status onion

# Default target
help:
	@echo "Quick Commands:"
	@echo "  up          - Start application"
	@echo "  down        - Stop application"
	@echo "  logs        - View logs"
	@echo "  status      - Check status"
	@echo "  onion       - Show .onion URL"
	@echo "  clean       - Removes all nonograph containers and backups"

# Production targets
build:
	@echo "Building Nonograph Docker images..."
	sudo docker compose build

up:
	@echo "Starting Nonograph application..."
	sudo docker compose up -d
	@echo "Application started! Access it at:"
	@echo "  Local: http://localhost:8009"
	@echo ""
	@echo "Getting Tor hidden service address..."
	@echo "Waiting for Tor to bootstrap (this may take 30-60 seconds)..."
	@sleep 45
	@echo "Your .onion address:"
	@sudo docker exec -u debian-tor nonograph_app cat /var/lib/tor/hidden_service/hostname 2>/dev/null || echo "Tor address not ready yet, check logs with 'make logs'"
	@echo ""
	@echo "Save this address to share your anonymous publishing platform!"

down:
	@echo "Stopping Nonograph application..."
	sudo docker compose down

logs:
	sudo docker compose logs -f

restart:
	@echo "Restarting Nonograph application..."
	sudo docker compose restart
	@echo "Application restarted!"

status:
	@echo "Container Status:"
	@echo "=================="
	sudo docker compose ps
	@echo ""
	@echo "Tor Hidden Service Address:"
	@echo "=========================="
	@sudo docker exec -u debian-tor nonograph_app cat /var/lib/tor/hidden_service/hostname 2>/dev/null || echo "No Tor address found"

# Development targets
dev:
	@echo "Starting Nonograph development environment..."
	sudo docker compose -f docker-compose.dev.yml up -d
	@echo "Development environment started!"
	@echo "  Application: http://localhost:8009"
	@echo "  Redis: localhost:6379"

dev-down:
	@echo "Stopping development environment..."
	sudo docker compose -f docker-compose.dev.yml down

dev-logs:
	sudo docker compose -f docker-compose.dev.yml logs -f

# Maintenance targets
clean:
	@echo "Cleaning up ALL Docker resources for Nonograph..."
	sudo docker compose down -v --remove-orphans 2>/dev/null || true
	sudo docker stop $$(sudo docker ps -aq) 2>/dev/null || true
	sudo docker rm $$(sudo docker ps -aq) 2>/dev/null || true
	sudo docker rmi $$(sudo docker images -q) 2>/dev/null || true
	sudo docker system prune -a -f --volumes
	sudo docker builder prune -a -f
	@echo "Complete cleanup finished - all Docker containers, images, volumes, and cache removed!"

# Show .onion address
onion:
	@echo "Your .onion address:"
	@sudo docker exec -u debian-tor nonograph_app cat /var/lib/tor/hidden_service/hostname 2>/dev/null || echo "No .onion address found"
