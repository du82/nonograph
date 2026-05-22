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

English | [简体中文](README.zh.md)

</div>

Self-hosted anonymous publishing. No accounts, no tracking. Write, publish, share. Nothing else collected.

https://github.com/user-attachments/assets/d662c9a2-f0ed-4266-bf55-e2c1f024269e

## Known Instances

| Uptime                                                                              | Location      | Clearnet                        | Onion                                                                  |
|-------------------------------------------------------------------------------------|---------------|---------------------------------|------------------------------------------------------------------------|
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonogra.ph)             | 🏴‍☠️ Unknown    | https://nonogra.ph              | http://ortmy3ey5usdzf4ivht6axtb72owjniaeqrexknosyons544aooltzyd.onion/ |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fwrite.daun.world)       | 🏴‍☠️ Unknown    | https://write.daun.world/       | http://fmoigm7j3z6vh4hgssdfhlt6knkp443thgxpe5wmbaevvb5km2d3suyd.onion/ |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fproxy.write.daun.world) | 🇫🇮 Finland    | https://proxy.write.daun.world/ | see above                                                              |
| Onion                                                                               | 🇭🇺 Hungary    |                                 | http://t7fgh7qvjysh3wer747m6dkjvkjsqvajyv5bh2grzjgpd2derxsxbdad.onion/ |
| Onion                                                                               | 🏴‍☠️ Unknown    |                                 | http://uawaa47jvsfr3ij63ns25xp6qvhqswsx3fgij2evbrcnt3ygxq3dbwyd.onion/ |
| Onion                                                                               | 🇰🇿 Kazakhstan |                                 | http://5mq3db45agipsceghnpx3iumlctya3absmp4sgnitqcmrmhaqhbbjcid.onion/ |

These instances are provided by third-parties, each with their own policies. Choose one that reflects your values, or self-host.

## Deploy

```bash
mkdir -p ~/nonograph/content ~/nonograph/onion
sudo docker run -d \
  --name nonograph \
  -p 8009:8009 \
  -v ~/nonograph/content:/app/content \
  -v ~/nonograph/onion:/var/lib/tor/hidden_service \
  --restart unless-stopped \
  ghcr.io/du82/nonograph:latest
```

or grab the source code and make your own container:

```bash
git clone https://github.com/du82/nonograph
cd nonograph
make up
```

Then check logs for your `.onion` address:

```bash
docker logs nonograph
```

Hate Docker? Run `./run` to build and run natively (Debian only).

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

## Audits
* 4/20/2026 - [@h_2_o0](https://t.me/h_2_o0) found a URL validation bypass on 4/20 (nice), fixed in [this commit](https://github.com/du82/nonograph/commit/639f64f010e2b287bf3429af1814dd4fb8697a16).
* 10/15/2025 - [Security Assessment Report Redacted.pdf](https://github.com/user-attachments/files/27242849/Security.Assessment.Report.Redacted.pdf) - audit of the initial release (v0.0.1), paid for in Monero. Only the auditors name and email were redacted. Fixed in [this](https://github.com/du82/nonograph/commit/2641fcaed1aaf458e69217e5489a75c93446b0d2) and [this](https://github.com/du82/nonograph/commit/98178a380324270da704aa80e035aea012e6e748) commit.


## License
Public domain ([Unlicense](https://unlicense.org)). This software belongs to everyone. Use it, modify it, share it without restriction. No attribution required, no strings attached, no warranties provided.
