<div align="center">
  <img align="center" width="96" height="96" alt="android-chrome-512x512" src="https://github.com/user-attachments/assets/9a06a3fe-46ee-422c-93ad-ce3e504603c0" />
</div>

<h1 align="center">Nonograph</h1>

<p align="center"><b>Anonymous publishing for the privacy-conscious web.</b></p>
<div align="center">
  <a href="https://unlicense.org">
    <img alt="GitHub License" src="https://img.shields.io/github/license/du82/nonograph">
  </a>
  <a href="https://github.com/du82/nonograph/releases/latest">
    <img alt="GitHub Release" src="https://img.shields.io/github/v/release/du82/nonograph">
  </a>
  <a href="https://github.com/du82/nonograph/commits/main/">
    <img alt="GitHub commit activity" src="https://img.shields.io/github/commit-activity/w/du82/nonograph">
  </a>
  <a href="http://aue5jcgehi2uq5gdrxuhfqmyw4yfrsq3ggd7bvcydqyhlnwha27iqiad.onion/">
    <img src="https://img.shields.io/badge/Tor-Hidden%20Service-7d4698?style=flat&logo=torproject&logoColor=white" alt="Tor Hidden Service">
  </a>
</div>

Self-hosted anonymous publishing. No accounts, no tracking. Write, publish, share. Nothing else collected.

https://github.com/user-attachments/assets/d662c9a2-f0ed-4266-bf55-e2c1f024269e

## Known Instances

| Uptime | Location | Clearnet | Onion |
|--------|----------|----------|-------|
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonogra.ph) | 🏴‍☠️ Unknown | https://nonogra.ph | http://aue5jcgehi2uq5gdrxuhfqmyw4yfrsq3ggd7bvcydqyhlnwha27iqiad.onion/ |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fwrite.daun.world) | 🏴‍☠️ Unknown | https://write.daun.world/ | http://fmoigm7j3z6vh4hgssdfhlt6knkp443thgxpe5wmbaevvb5km2d3suyd.onion/ |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fproxy.write.daun.world) | 🇫🇮 Finland | https://proxy.write.daun.world/ | see above |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnull) | 🇰🇿 Kazakhstan | | http://5mq3db45agipsceghnpx3iumlctya3absmp4sgnitqcmrmhaqhbbjcid.onion/ |

## Deploy

```bash
git clone https://github.com/du82/nonograph
cd nonograph
make up
```

`make up` installs Docker if needed, builds the container with Tor included, and prints your `.onion` address.

```bash
make up        # Start the service on Tor
make tor       # Print .onion address
make status    # Check status
make down      # Stop containers
make clean     # Remove container entirely
```
Hate Docker? Run `./run` to build and run natively (Linux only).

## Features
- Markdown with tables, code blocks, footnotes, and `#spoiler#` syntax
- Image and video embedding from URLs
- No accounts, no IPs logged, no analytics
- Tor hidden service out of the box
- Runs on 64MB RAM. Fine on a Pi or cheap VPS

## Requirements
- Debian-based Linux (Raspberry Pi OS, KDE Neon, Pop_OS, etc. - not Ubuntu)
- 64MB RAM, 64MB disk

## Name
`anonymous` + `monograph` + `telegraph` = `nonograph`

## License
Public domain ([Unlicense](https://unlicense.org)). This software belongs to everyone. Use it, modify it, share it without restriction. No attribution required, no strings attached, no warranties provided.
