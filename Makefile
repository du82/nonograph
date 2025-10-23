.PHONY: help build up down logs clean dev dev-down dev-logs test restart status onion check-docker install-docker remove-docker

# Default target
help:
	@echo "Quick Commands:"
	@echo "  up          - Start application (checks for Docker first)"
	@echo "  down        - Stop application"
	@echo "  logs        - View logs"
	@echo "  status      - Check status"
	@echo "  onion       - Show .onion URL"
	@echo "  clean       - Removes all nonograph containers and backups"
	@echo ""
	@echo "Docker Management:"
	@echo "  check-docker     - Check if Docker is installed"
	@echo "  install-docker   - Install Docker on this system"
	@echo "  remove-docker    - Completely remove Docker from this system"
	@echo ""
	@echo "Note: All commands will automatically check for Docker and offer to install it if missing."

# Check if Docker and Docker Compose are installed
check-docker:
	@echo "Checking Docker installation..."
	@if ! command -v docker >/dev/null 2>&1; then \
		echo "❌ Docker is not installed on this system."; \
		echo "You need Docker to run Nonograph. Would you like to install it? (y/N)"; \
		read -r response; \
		if [ "$$response" = "y" ] || [ "$$response" = "Y" ]; then \
			$(MAKE) install-docker; \
		else \
			echo "Docker installation cancelled. Please install Docker manually and try again."; \
			echo "Visit: https://docs.docker.com/get-docker/"; \
			exit 1; \
		fi; \
	else \
		echo "✅ Docker is installed"; \
	fi
	@if ! command -v docker compose >/dev/null 2>&1 && ! command -v docker-compose >/dev/null 2>&1; then \
		echo "❌ Docker Compose is not installed on this system."; \
		echo "You need Docker Compose to run Nonograph. Would you like to install it? (y/N)"; \
		read -r response; \
		if [ "$$response" = "y" ] || [ "$$response" = "Y" ]; then \
			$(MAKE) install-docker-compose; \
		else \
			echo "Docker Compose installation cancelled. Please install Docker Compose manually and try again."; \
			echo "Visit: https://docs.docker.com/compose/install/"; \
			exit 1; \
		fi; \
	else \
		echo "✅ Docker Compose is installed"; \
	fi

# Install Docker (works on most Linux distributions)
install-docker:
	@echo "Installing Docker..."
	@echo "Using Docker's official installation script for maximum compatibility..."; \
	curl -fsSL https://get.docker.com -o get-docker.sh && \
	sudo sh get-docker.sh && \
	rm get-docker.sh && \
	sudo usermod -aG docker $$USER && \
	if ! command -v docker compose >/dev/null 2>&1; then \
		echo "Installing Docker Compose plugin..."; \
		sudo apt update && sudo apt install -y docker-compose-plugin 2>/dev/null || \
		sudo yum install -y docker-compose-plugin 2>/dev/null || \
		sudo pacman -S --noconfirm docker-compose 2>/dev/null || \
		$(MAKE) install-docker-compose; \
	fi && \
	echo "✅ Docker installed successfully! Please log out and back in to use Docker without sudo."

# Install Docker Compose (fallback for systems without compose plugin)
install-docker-compose:
	@echo "Installing Docker Compose..."
	@sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$$(uname -s)-$$(uname -m)" -o /usr/local/bin/docker-compose
	@sudo chmod +x /usr/local/bin/docker-compose
	@echo "✅ Docker Compose installed successfully!"

# Production targets
build: check-docker
	@echo "Building Nonograph Docker images..."
	sudo docker compose build

up: check-docker
	@echo "Starting Nonograph application..."
	sudo docker compose up -d
	@echo "Application started! Access it at:"
	@echo "  Local: http://localhost:8009"
	@echo ""
	@echo "Getting Tor hidden service address..."
	@echo "Waiting for Tor to bootstrap (this may take 30-60 seconds)..."
	@sleep 30
	@echo "Your .onion address:"
	@sudo docker exec -u debian-tor nonograph_app cat /var/lib/tor/hidden_service/hostname 2>/dev/null || echo "Tor address not ready yet, check logs with 'make logs'"
	@echo ""
	@echo "Save this address to share your anonymous publishing platform!"

down: check-docker
	@echo "Stopping Nonograph application..."
	sudo docker compose down

logs: check-docker
	sudo docker compose logs -f

restart: check-docker
	@echo "Restarting Nonograph application..."
	sudo docker compose restart
	@echo "Application restarted!"

status: check-docker
	@echo "Container Status:"
	@echo "=================="
	sudo docker compose ps
	@echo ""
	@echo "Tor Hidden Service Address:"
	@echo "=========================="
	@sudo docker exec -u debian-tor nonograph_app cat /var/lib/tor/hidden_service/hostname 2>/dev/null || echo "No Tor address found"

# Development targets
dev: check-docker
	@echo "Starting Nonograph development environment..."
	sudo docker compose -f docker-compose.dev.yml up -d
	@echo "Development environment started!"
	@echo "  Application: http://localhost:8009"
	@echo "  Redis: localhost:6379"

dev-down: check-docker
	@echo "Stopping development environment..."
	sudo docker compose -f docker-compose.dev.yml down

dev-logs: check-docker
	sudo docker compose -f docker-compose.dev.yml logs -f

# Maintenance targets
clean: check-docker
	@echo "Cleaning up ALL Docker resources for Nonograph..."
	sudo docker compose down -v --remove-orphans 2>/dev/null || true
	sudo docker stop $$(sudo docker ps -aq) 2>/dev/null || true
	sudo docker rm $$(sudo docker ps -aq) 2>/dev/null || true
	sudo docker rmi $$(sudo docker images -q) 2>/dev/null || true
	sudo docker system prune -a -f --volumes
	sudo docker builder prune -a -f
	@echo "Complete cleanup finished - all Docker containers, images, volumes, and cache removed!"

# Remove Docker completely from the system
remove-docker:
	@echo "⚠️  WARNING: This will completely remove Docker and all containers, images, and data!"
	@echo "Are you sure you want to proceed? (y/N)"
	@read -r response; \
	if [ "$$response" = "y" ] || [ "$$response" = "Y" ]; then \
		echo "Stopping all Docker containers..."; \
		sudo docker stop $$(sudo docker ps -aq) 2>/dev/null || true; \
		echo "Removing all Docker containers..."; \
		sudo docker rm $$(sudo docker ps -aq) 2>/dev/null || true; \
		echo "Removing all Docker images..."; \
		sudo docker rmi $$(sudo docker images -q) 2>/dev/null || true; \
		echo "Removing all Docker volumes..."; \
		sudo docker volume rm $$(sudo docker volume ls -q) 2>/dev/null || true; \
		echo "Removing all Docker networks..."; \
		sudo docker network rm $$(sudo docker network ls -q) 2>/dev/null || true; \
		echo "Purging Docker system..."; \
		sudo docker system prune -a -f --volumes 2>/dev/null || true; \
		echo "Stopping Docker service..."; \
		sudo systemctl stop docker 2>/dev/null || true; \
		sudo systemctl stop containerd 2>/dev/null || true; \
		sudo systemctl disable docker 2>/dev/null || true; \
		sudo systemctl disable containerd 2>/dev/null || true; \
		if command -v apt >/dev/null 2>&1; then \
			echo "Removing Docker packages (Debian/Ubuntu)..."; \
			sudo apt purge -y docker-ce docker-ce-cli containerd.io docker-compose-plugin docker-ce-rootless-extras 2>/dev/null || true; \
			sudo apt purge -y docker.io docker-compose 2>/dev/null || true; \
			sudo apt autoremove -y; \
			sudo apt autoclean; \
		elif command -v yum >/dev/null 2>&1; then \
			echo "Removing Docker packages (RHEL/CentOS/Fedora)..."; \
			sudo yum remove -y docker-ce docker-ce-cli containerd.io docker-compose-plugin 2>/dev/null || true; \
			sudo yum remove -y docker docker-common docker-selinux docker-engine 2>/dev/null || true; \
		elif command -v pacman >/dev/null 2>&1; then \
			echo "Removing Docker packages (Arch Linux)..."; \
			sudo pacman -Rns --noconfirm docker docker-compose 2>/dev/null || true; \
		fi; \
		echo "Removing Docker directories and files..."; \
		sudo rm -rf /var/lib/docker 2>/dev/null || true; \
		sudo rm -rf /var/lib/containerd 2>/dev/null || true; \
		sudo rm -rf /etc/docker 2>/dev/null || true; \
		sudo rm -rf /etc/systemd/system/docker.service.d 2>/dev/null || true; \
		sudo rm -f /etc/apt/sources.list.d/docker.list 2>/dev/null || true; \
		sudo rm -f /etc/apt/keyrings/docker.gpg 2>/dev/null || true; \
		sudo rm -f /usr/local/bin/docker-compose 2>/dev/null || true; \
		echo "Removing Docker group..."; \
		sudo groupdel docker 2>/dev/null || true; \
		echo "Removing user from Docker group..."; \
		sudo gpasswd -d $$USER docker 2>/dev/null || true; \
		echo "Reloading systemd daemon..."; \
		sudo systemctl daemon-reload 2>/dev/null || true; \
		echo "✅ Docker has been completely removed from your system!"; \
		echo "You may want to log out and back in to refresh group memberships."; \
	else \
		echo "Docker removal cancelled."; \
	fi

# Show .onion address
onion: check-docker
	@echo "Your .onion address:"
	@sudo docker exec -u debian-tor nonograph_app cat /var/lib/tor/hidden_service/hostname 2>/dev/null || echo "No .onion address found"
