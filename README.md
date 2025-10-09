# Nonograph

**Anonymous publishing for the privacy-conscious web.**

Nonograph is a self-hosted anonymous publishing platform. No accounts, no tracking, no data collection. Write anonymously, publish instantly, read privately.

## One-Command Deploy

```bash
./run
```

This single command:
- Installs all dependencies (Rust, Tor)
- Configures Tor hidden service with persistent `.onion` address
- Builds and launches Nonograph
- Works on any Debian system (NOT Ubuntu)

## Why Self-Host?

**Complete Control**: You own your platform, your content, your readers' privacy.

**True Anonymity**: No third-party servers logging IPs, no corporate surveillance.

**Censorship Resistant**: Your `.onion` address can't be blocked or seized.

**Zero Dependencies**: No external APIs or services. Free and public domain forever.

## Privacy by Design

- **No user tracking** - We don't collect IP addresses, user agents, or analytics
- **No accounts** - Write and publish without identity
- **Tor-native** - Built-in hidden service support
- **Local storage** - All data stays on your server
- **Open source** - Audit the privacy claims yourself

## For Writers

- **Instant publishing** - Write, click publish, get shareable link
- **Rich markdown** - Headers, tables, code blocks, footnotes and a few extras
- **Secret text** - #Click to reveal# spoiler functionality
- **Media embedding** - Images and videos from URLs
- **Clean interface** - Distraction-free writing experience
- **Anonymous by default** - Nothing collected beyond the words you type

## For Readers

- **Fast loading** - Lightweight, optimized for speed and readability
- **No tracking** - Read without surveillance, no one can
- **Mobile friendly** - Works on any device
- **Tor accessible** - Read via `.onion` addresses
- **Share freely** - Links work on both clearnet and Tor

## Access Your Platform

After running `./run`:
- **Local**: `http://localhost:8009`
- **Tor**: Your persistent `.onion` address (shown in terminal)
- **Persistent**: Same onion address survives restarts and reboots

## Management

```bash
./run status    # Show onion address and service status
./run stop      # Stop the service
./run restart   # Restart the service
./run logs      # View logs
```

## System Requirements

- Any Debian-based Linux (Raspberry Pi OS, KDE Neon, Pop_OS, etc.)
- 512MB RAM minimum
- 100MB disk space
- Internet connection for initial setup

Runs perfectly on a Raspberry Pi or cheap VPS.

## Security Features

- Content stored as plain files (no database to compromise)
- Automatic Tor hidden service configuration
- HTML sanitization prevents XSS attacks
- No user-uploaded files (security by design)
- Minimal attack surface (single Rust binary)
- No external network calls after setup

---

**License**: Public Domain ([Unlicense](https://unlicense.org))

This software belongs to everyone. Use it, modify it, share it without restriction. No attribution required, no strings attached, no warranties provided.
