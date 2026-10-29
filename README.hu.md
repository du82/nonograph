<div align="center">
  <img align="center" width="96" height="96" alt="android-chrome-512x512" src="https://github.com/user-attachments/assets/9a06a3fe-46ee-422c-93ad-ce3e504603c0" />
</div>

<h1 align="center">Nonograph</h1>

<p align="center"><b>Anonim publikáció a magánszféra-tudatos net számára.</b></p>
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

[English](README.md) | [简体中文](README.zh.md) | [Русский](README.ru.md) | Magyar

</div>

A Nonograph egy egyszerű anonim publikációs platform bárkinek aki szeretné, hogy szavai online legyenek egy hozzákapcsolt személyazonosság nélkül. Írsz valamit, kapsz egy megosztható linket, és kész. Nincs felhasználói fiók, nincs profil, és nincs elemzés ami követné az olvasókat.

https://github.com/user-attachments/assets/d662c9a2-f0ed-4266-bf55-e2c1f024269e

## Ismert szerverek

| Elérhetőség | Földrajzi hely | Clearnet | Onion | |
|--------------------------------------------------------------------------------------|---------------|----------------------------------|------------------------------------------------------------------------|---|
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonogra.ph) | 🏴‍☠️ Ismeretlen | https://nonogra.ph | http://ortmy3ey5usdzf4ivht6axtb72owjniaeqrexknosyons544aooltzyd.onion/ | |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fwrite.eversiege.network) | 🏴‍☠️ Ismeretlen | https://write.eversiege.network/ | http://fmoigm7j3z6vh4hgssdfhlt6knkp443thgxpe5wmbaevvb5km2d3suyd.onion/ | |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonograph.com) | 🇭🇰 Hong Kong | https://nonograph.com<br>https://nonograph.ink | http://gt65bmujun7alps7b7oar5x2u5lprxjdvpwrxcrnwkgheocjwllchhqd.onion/ | |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fproxy.write.daun.world) | 🇫🇮 Finnország | https://proxy.write.daun.world/ | see above | |
| Onion | 🇷🇺 Oroszország | | http://q2w7sdjlmfc5vjif6y3372665wzyqskqvjji7bmhfn72orxcbljvonid.onion/ | |
| Onion | 🇭🇺 Magyarország | | http://oahjeuod7h7kiu7nfhasx34djvnwtlgzogk7a5t7kl2gqskskhi4r5qd.onion/ | |
| Onion | 🏴‍☠️ Ismeretlen | | http://uawaa47jvsfr3ij63ns25xp6qvhqswsx3fgij2evbrcnt3ygxq3dbwyd.onion/ | |
| Onion | 🇰🇿 Kazahsztán | | http://5mq3db45agipsceghnpx3iumlctya3absmp4sgnitqcmrmhaqhbbjcid.onion/ | |

Ezeket a szervereket harmadik felek üzemeltetik, mind a saját szabályaikkal. Válassz egyet ami tükrözi az elveidet, vagy hostolj magadnak.

## Hostolás

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

vagy töltsd le a forráskódot és csinálj saját containert:

```bash
git clone https://github.com/du82/nonograph
cd nonograph
make up
```

Majd nézd meg a logokat az`.onion` címedért:

```bash
docker logs nonograph
```

Utálod a Dockert? Futtasd a  `./run`-t hogy helyben buildeld és működtetsd a programot (csak Debian).

## Funkciók
A Nonograph számos formázási lehetőséget ad,  nyomd le a `/` billentyűt egy új sorban, hogy listát kapj.
<img width="561" height="447" alt="image" src="https://github.com/user-attachments/assets/cda96a9c-08bc-4add-bf5b-e8eb0b352201" />


## Képernyőfotók
A kezdőlap és a szövegmező:

<img width="1920" height="1080" alt="homepage" src="https://github.com/user-attachments/assets/d77c065d-a02f-40f5-b29f-fe45465af018" />

A szerkesztő egy félkész oldallal:

<img width="1920" height="1080" alt="editor" src="https://github.com/user-attachments/assets/7546a84f-b172-4c0d-a20a-df6c2defbf3c" />

Egy közzétett oldal képpel:

<img width="1920" height="1080" alt="published-page2" src="https://github.com/user-attachments/assets/0fc38a43-8bcc-4fbf-9087-ea4100be3e6c" />

## Itt is elérhető:
- Radicle: [`rad:z2irFqQ3Zx1S8kqsVgH9itBoVUxkT`](https://radicle.network/nodes/iris.radicle.network/rad%3Az2irFqQ3Zx1S8kqsVgH9itBoVUxkT)

## Elnevezés és a kezdetek
`anonymous` + `monograph` + `telegraph` = `nonograph`

A Nonograph egy nyílt forráskódú Telegraph (telegra.ph) alternatíva az anonimitással és az adatvédelemmel fókuszban.  [Ez az oldal](https://nonogra.ph/history-of-this-10-29-2026) jól összefoglalja a történetét.

## Auditok és biztonság

- 2026/10/13  - [@du82](https://github.com/du82) elérhetővé tett 13200 USD értékű Monerot a bug bounty programhoz.
- 2026/5/9 - [@netqo](https://github.com/netqo) biztonsági fejlesztéseket tett. Moneroban kifizetve [itt](https://github.com/du82/nonograph/pull/28).
* 8/21/2026 - [@sgpinkus](https://github.com/sgpinkus) biztonsági fejlesztéseket tett. Moneroban kifizetve [itt](https://github.com/du82/nonograph/pull/26).
* 6/22/2026 - [@SmokeCamel](t.me/cigssss) költségei meg lettek térítve, ls elkülönített virtuális gépeket kapott, hogy a legújabb AI modelleket futtassa a Nonograph parsere ellen. Sérülékenységekre nem derült fény.
* 4/20/2026 - [@h_2_o0](https://t.me/h_2_o0) talált egy URL validációs hibát, javítva [itt](https://github.com/du82/nonograph/commit/639f64f010e2b287bf3429af1814dd4fb8697a16).
* 10/15/2025 - [Security Assessment Report Redacted.pdf](https://github.com/user-attachments/files/27242849/Security.Assessment.Report.Redacted.pdf) - az első kiadás (v0.0.1) auditja, Moneroval fizetve. Csak a szerző neve és e-mail címe lett kitakarva. Javítva [itt](https://github.com/du82/nonograph/commit/2641fcaed1aaf458e69217e5489a75c93446b0d2) és [itt](https://github.com/du82/nonograph/commit/98178a380324270da704aa80e035aea012e6e748).

## Licenc
Public domain ([Unlicense](https://unlicense.org)). Ez a szoftver mindenkié. Használd, módosítsd, oszd meg másokkal korlátozás nélkül. Csinálj vele amit szeretnél, de ne feledd, nincs semmilyen garancia. Tartsd tiszteletben a mások által hostolt Nonograph szerverek szabályait.
