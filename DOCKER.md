# Docker Setup

## Install Docker

```bash
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh
sudo usermod -aG docker $USER
```

## Run

```bash
make up        # Start
make tor       # Get .onion address
make status    # Check status
make down      # Stop
make clean     # Completely remove container
```

Access: http://localhost:8009
