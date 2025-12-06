
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
  <a href="http://5mq3db45agipsceghnpx3iumlctya3absmp4sgnitqcmrmhaqhbbjcid.onion/">
    <img src="https://img.shields.io/badge/Tor-Hidden%20Service-7d4698?style=flat&logo=torproject&logoColor=white" alt="Tor Hidden Service">
  </a>
</div>

Nonograph is a self-hosted anonymous publishing platform. No accounts, no tracking, no compromise. Write anonymously, publish instantly, read privately.

https://github.com/user-attachments/assets/d662c9a2-f0ed-4266-bf55-e2c1f024269e

## Known Instances
| Avg. Uptime | Region        | Clearnet           | Onion                                                                  |
|-------------|---------------|--------------------|------------------------------------------------------------------------|
| 100%        | 🏴‍☠️ Unknown    | https://nonogra.ph | http://aue5jcgehi2uq5gdrxuhfqmyw4yfrsq3ggd7bvcydqyhlnwha27iqiad.onion/ |
| 100%        | 🇰🇿 Kazakhstan |                    | http://5mq3db45agipsceghnpx3iumlctya3absmp4sgnitqcmrmhaqhbbjcid.onion/ |
| 85%         | 🏴‍☠️ Unknown    |                    | http://mlugttjcd6tmcigby3owjelkt6mpxqjzwq2hd3vrub7uok3sl6ms2cid.onion/ |

## Deploy as a hidden service on Tor with Docker!

```bash
git clone https://github.com/du82/nonograph
cd nonograph
make up
```

This single `make up` command:
- Installs Docker if it's not installed
- Spins up a Debian container with the project, Tor, and all security taken care of
- Starts the service and provides you with an onion address

Your brand new hidden service can be up and running in under a minute with a single command!

Or, run `make` to view all options:

```bash
make up        # Start the service on Tor
make tor       # Get .onion address
make status    # Check status
make down      # Stop all nonograph containers
make clean     # Completely remove container
```

If you prefer, you can run `./run` to compile and run without docker

## Why Self-Host?

**Complete Control**: You own your platform, your content, your readers' privacy.

**True Anonymity**: No third-party servers logging IPs, no corporate surveillance.

**Censorship Resistant**: Your `.onion` address can't be blocked or seized without considerable effort.

**Zero Rentseekers**: No external APIs or services. Free and public domain forever.

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

## System Requirements

- Any Debian-based Linux (Raspberry Pi OS, KDE Neon, Pop_OS, etc. NOT Ubuntu)
- 64MB RAM minimum
- 64MB disk space minimum
- Internet connection if you wish to host on Tor

Runs perfectly on a Raspberry Pi or cheap VPS.

## Naming rationale
`anonymous` + `monograph` + `telegraph` = `nonograph`
* Pseudonymity is the use of a fictitious name or alias instead of a person's real name, often to protect their identity and privacy while engaging in various activities, especially online.
* A monograph is generally a long-form work on one subject, or one aspect of a subject, typically created by a single author or artist.
* Telegraphy is the long-distance transmission of messages where the sender uses symbolic codes, known to the recipient, rather than a physical exchange of an object bearing the message.

Therefore, a Nonograph would be:
* Nonograph (noun): A pseudonymous or anonymous written work, typically long-form, transmitted remotely without physical exchange and designed to protect the author's identity.

---

**License**: Public Domain ([Unlicense](https://unlicense.org))

This software belongs to everyone. Use it, modify it, share it without restriction. No attribution required, no strings attached, no warranties provided.
