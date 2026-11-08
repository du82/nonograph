<div align="center">
  <img align="center" width="96" height="96" alt="android-chrome-512x512" src="https://github.com/user-attachments/assets/9a06a3fe-46ee-422c-93ad-ce3e504603c0" />
</div>

<h1 align="center">Nonograph</h1>

<p align="center"><b>Publicación anónima para una web que respeta la privacidad.</b></p>
<div align="center">
  <a href="https://unlicense.org">
    <img alt="GitHub License" src="https://img.shields.io/github/license/du82/nonograph">
  </a>
  <a href="https://github.com/du82/nonograph/releases/latest">
    <img alt="GitHub Release" src="https://img.shields.io/github/v/release/du82/nonograph">
  </a>
  <a href="https://github.com/du82/nonograph/commits/main/">
    <img alt="GitHub commit activity" src="https://img.shields.io/github/commit-activity/m/du82/nonograph">
  </a>
  <a href="http://ortmy3ey5usdzf4ivht6axtb72owjniaeqrexknosyons544aooltzyd.onion/">
    <img src="https://img.shields.io/badge/Tor-Hidden%20Service-7d4698?style=flat&logo=torproject&logoColor=white" alt="Tor Hidden Service">
  </a>

[English](README.md) | [简体中文](README.zh.md) | [Русский](README.ru.md) | Español | [Português (BR)](README.pt-BR.md) | [Français](README.fr.md) | [Magyar](README.hu.md) | [Türkçe](README.tr.md)

</div>

Nonograph es una plataforma de publicación sencilla y anónima para cualquiera que quiera que sus palabras estén disponibles en línea sin estar vinculadas a una identidad. Escribes, obtienes un enlace para compartir y eso es todo. No hay que crear una cuenta, no hay perfil ni estadísticas que rastreen a los lectores.

https://github.com/user-attachments/assets/d662c9a2-f0ed-4266-bf55-e2c1f024269e

## Instancias Conocidas

| Uptime                                                                               | Location      | Clearnet                         | Onion                                                                  |   |
|--------------------------------------------------------------------------------------|---------------|----------------------------------|------------------------------------------------------------------------|---|
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonogra.ph)              | 🏴‍☠️ Unknown    | https://nonogra.ph               | http://ortmy3ey5usdzf4ivht6axtb72owjniaeqrexknosyons544aooltzyd.onion/ |   |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fwrite.eversiege.network) | 🏴‍☠️ Unknown    | https://write.eversiege.network/ | http://fmoigm7j3z6vh4hgssdfhlt6knkp443thgxpe5wmbaevvb5km2d3suyd.onion/ |   |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonograph.com)           | 🇭🇰 Hong Kong  | https://nonograph.com<br>https://nonograph.ink            | http://gt65bmujun7alps7b7oar5x2u5lprxjdvpwrxcrnwkgheocjwllchhqd.onion/ |   |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fproxy.write.daun.world)  | 🇫🇮 Finland    | https://proxy.write.daun.world/  | see above                                                              |   |
| Onion                                                                                | 🇷🇺 Russia     |                                  | http://q2w7sdjlmfc5vjif6y3372665wzyqskqvjji7bmhfn72orxcbljvonid.onion/ |   |
| Onion                                                                                | 🇭🇺 Hungary    |                                  | http://oahjeuod7h7kiu7nfhasx34djvnwtlgzogk7a5t7kl2gqskskhi4r5qd.onion/ |   |
| Onion                                                                                | 🏴‍☠️ Unknown    |                                  | http://uawaa47jvsfr3ij63ns25xp6qvhqswsx3fgij2evbrcnt3ygxq3dbwyd.onion/ |   |
| Onion                                                                                | 🇰🇿 Kazakhstan |                                  | http://5mq3db45agipsceghnpx3iumlctya3absmp4sgnitqcmrmhaqhbbjcid.onion/ |   |

Estas instancias son proporcionadas por terceros, cada una con sus propias políticas. Elige una que refleje tus valores o autoalójala.

## Instalacion

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

o descarga el código fuente y crea tu propio contenedor:

```bash
git clone https://github.com/du82/nonograph
cd nonograph
make up
```

Luego, Comprueba Los Registros De Tu `.onion` Direccion:

```bash
docker logs nonograph
```

Hate Docker? Run `./run` to build and run natively (Debian only).

## Caracterisicas
Nonograph incluye una amplia lista de opciones de marcado; escribe `/` en una nueva línea para ver la lista.

<img width="561" height="447" alt="image" src="https://github.com/user-attachments/assets/cda96a9c-08bc-4add-bf5b-e8eb0b352201" />


## Capturas de Pantalla
The homepage and writing area:

<img width="1920" height="1080" alt="homepage" src="https://github.com/user-attachments/assets/d77c065d-a02f-40f5-b29f-fe45465af018" />

The editor with a page in progress:

<img width="1920" height="1080" alt="editor" src="https://github.com/user-attachments/assets/7546a84f-b172-4c0d-a20a-df6c2defbf3c" />

A published page with an image:

<img width="1920" height="1080" alt="published-page2" src="https://github.com/user-attachments/assets/0fc38a43-8bcc-4fbf-9087-ea4100be3e6c" />

## Tambien disponible en...
- Radicle: [`rad:z2irFqQ3Zx1S8kqsVgH9itBoVUxkT`](https://radicle.network/nodes/iris.radicle.network/rad%3Az2irFqQ3Zx1S8kqsVgH9itBoVUxkT)

## Nombres Y Origenes
`anonymous` + `monograph` + `telegraph` = `nonograph`

Nonograph es una alternativa de código abierto a Telegraph (telegra.ph) centrada en la privacidad y el anonimato. Aquí hay una [historia del proyecto](https://nonogra.ph/history-of-this-10-29-2026) bastante completa que alguien elaboró con la intención de revelar mi identidad.

«Nonograph» se pronuncia como «non» + «oh» + «graph». «Non», como en «aNONymous».

## Auditorías y seguridad

- 10/13/2026 - [@du82](https://github.com/du82) added $13,200 USD worth of Monero to the bug bounty program.
- 9/5/2026 - [@netqo](https://github.com/netqo) contributed security improvements. Paid in Monero in [this pull request](https://github.com/du82/nonograph/pull/28).
* 8/21/2026 - [@sgpinkus](https://github.com/sgpinkus) contributed security improvements. Paid in Monero in [this pull request](https://github.com/du82/nonograph/pull/26).
* 6/22/2026 - [@SmokeCamel](t.me/cigssss) was reimbursed and provided isolated VMs for running top AI models against Nonograph's parser. No vulnerabilities were found.
* 4/20/2026 - [@h_2_o0](https://t.me/h_2_o0) found a URL validation bypass on 4/20 (nice), fixed in [this commit](https://github.com/du82/nonograph/commit/639f64f010e2b287bf3429af1814dd4fb8697a16).
* 10/15/2025 - [Security Assessment Report Redacted.pdf](https://github.com/user-attachments/files/27242849/Security.Assessment.Report.Redacted.pdf) - audit of the initial release (v0.0.1), paid for in Monero. Only the auditors name and email were redacted. Fixed in [this](https://github.com/du82/nonograph/commit/2641fcaed1aaf458e69217e5489a75c93446b0d2) and [this](https://github.com/du82/nonograph/commit/98178a380324270da704aa80e035aea012e6e748) commit.

## Licencias
Dominio público ([Unlicense](https://unlicense.org)). Este software pertenece a todo el mundo. Úsalo, modifícalo y compártelo sin restricciones. Haz lo que quieras con él, pero recuerda que no hay garantía alguna. Por favor, respeta las políticas de los servidores de Nonograph en los que te encuentres.

## Gracias
Nonograph ha sido traducido por personas al ruso, húngaro, español, portugués y ucraniano. Su seguridad está garantizada gracias a personas que revisan el código, detectan casos extremos inusuales y reciben una remuneración por sus hallazgos. Está alojado por personas en docenas de servidores repartidos por todo el mundo, que lo ponen a disposición de todos de forma gratuita. El logotipo fue diseñado por un artista al que se le pagó bien. Gracias a todas las personas que lo han hecho posible.
