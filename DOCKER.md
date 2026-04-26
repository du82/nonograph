# Docker Setup

## Install Docker

```bash
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh
sudo usermod -aG docker $USER
```

## Run with Docker Compose (recommended)

```bash
make up        # Start
make tor       # Get .onion address
make status    # Check status
make down      # Stop
make clean     # Completely remove container
```

Content is stored at `~/nonograph/content`.

Access: http://localhost:8009

## Run manually

```bash
mkdir -p ~/nonograph/content
sudo docker run -d \
  -p 8009:8009 \
  -v ~/nonograph/content:/app/content \
  --name nonograph \
  ghcr.io/du82/nonograph:latest
```

Then check logs for your `.onion` address:

```bash
sudo docker logs nonograph
```
