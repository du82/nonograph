# Docker Setup

## Install Docker

**Linux:**
```bash
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh
sudo usermod -aG docker $USER
```

**Windows:**
Download and install [Docker Desktop](https://www.docker.com/products/docker-desktop/).

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

**Linux/macOS:**
```bash
mkdir -p ~/nonograph/content ~/nonograph/onion
docker run -d \
  --name nonograph \
  -p 8009:8009 \
  -v ~/nonograph/content:/app/content \
  -v ~/nonograph/onion:/var/lib/tor/hidden_service \
  --restart unless-stopped \
  ghcr.io/du82/nonograph:latest
```

**Windows (PowerShell):**
```powershell
mkdir -Force $env:USERPROFILE\nonograph\content, $env:USERPROFILE\nonograph\onion
docker run -d `
  --name nonograph `
  -p 8009:8009 `
  -v $env:USERPROFILE\nonograph\content:/app/content `
  -v $env:USERPROFILE\nonograph\onion:/var/lib/tor/hidden_service `
  --restart unless-stopped `
  ghcr.io/du82/nonograph:latest
```

Then check logs for your `.onion` address:

```bash
docker logs nonograph
```
