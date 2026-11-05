<div align="center">
  <img align="center" width="96" height="96" alt="android-chrome-512x512" src="https://github.com/user-attachments/assets/9a06a3fe-46ee-422c-93ad-ce3e504603c0" />
</div>

<h1 align="center">Nonograph</h1>

<p align="center"><b>Publications anonymes pour un internet plus respectueux de la vie privée.</b></p>
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

[English](README.md) | [简体中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md) | Français | [Magyar](README.hu.md) | [Türkçe](README.tr.md)

</div>

Nonograph est une plateforme simple de publication anonyme pour tous ceux qui veulent publier leurs écrits en ligne sans qu’ils soient reliés à une identité. Vous écrivez, vous obtenez un lien partageable, et c’est tout. Aucun compte à créer, aucun profil, aucune statistique, et donc aucune trace qui vous poursuit.

https://github.com/user-attachments/assets/d662c9a2-f0ed-4266-bf55-e2c1f024269e

## Instances publiques

| Uptime |    Location   |                   Clearnet                  |                                  Onion                                 |
|:------:|:--------------|:--------------------------------------------|:-----------------------------------------------------------------------|
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonogra.ph)      | 🏴‍☠️ Unknown    | https://nonogra.ph                          | http://ortmy3ey5usdzf4ivht6axtb72owjniaeqrexknosyons544aooltzyd.onion/ |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fwrite.eversiege.network)      | 🏴‍☠️ Unknown    | https://write.eversiege.network/            | http://fmoigm7j3z6vh4hgssdfhlt6knkp443thgxpe5wmbaevvb5km2d3suyd.onion/ |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fwrite.anarchists.space)      | 🏴‍☠️ Unknown    | https://write.anarchists.space/             | http://qrb7g7gnowbpxxbpjeqaxr3a6odcvipd2xdumemvqj2nyfusghzzv6yd.onion  |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonograph.com)      | 🇭🇰 Hong Kong  | https://nonograph.com https://nonograph.ink | http://gt65bmujun7alps7b7oar5x2u5lprxjdvpwrxcrnwkgheocjwllchhqd.onion/ |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fdisc.us.ci)      | 🏴‍☠️ Unknown    | https://disc.us.ci/             | Tor unreachable                                                              |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fproxy.write.daun.world)      | 🇫🇮 Finland    | https://proxy.write.daun.world/             | http://fmoigm7j3z6vh4hgssdfhlt6knkp443thgxpe5wmbaevvb5km2d3suyd.onion/ |
| Onion  | 🇷🇺 Russia     |                                             | http://q2w7sdjlmfc5vjif6y3372665wzyqskqvjji7bmhfn72orxcbljvonid.onion/ |
| Onion  | 🇭🇺 Hungary    |                                             | http://oahjeuod7h7kiu7nfhasx34djvnwtlgzogk7a5t7kl2gqskskhi4r5qd.onion/ |
| Onion  | 🏴‍☠️ Unknown    |                                             | http://uawaa47jvsfr3ij63ns25xp6qvhqswsx3fgij2evbrcnt3ygxq3dbwyd.onion/ |
| Onion  | 🇰🇿 Kazakhstan |                                             | http://5mq3db45agipsceghnpx3iumlctya3absmp4sgnitqcmrmhaqhbbjcid.onion/ |

Ces instances sont proposées par des tiers, chacun avec leurs propres règles. Choisissez celle qui correspond à vos valeurs, ou hébergez-en une vous-même.

## Héberger

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

Ou récupérez le code source et créez votre propre conteneur :

```bash
git clone https://github.com/du82/nonograph
cd nonograph
make up
```

Vérifiez ensuite les logs pour votre adresse `.onion` :

```bash
docker logs nonograph
```

Vous détestez Docker ? Exécutez `./run` pour compiler et exécuter Nonograph nativement (Debian seulement).

## Fonctionnalités
De nombreuses options de balisage sont disponibles dans Nonograph ; appuyez sur `/` dans une nouvelle ligne pour en afficher une liste.

<img width="561" height="447" alt="image" src="https://github.com/user-attachments/assets/cda96a9c-08bc-4add-bf5b-e8eb0b352201" />


## Captures d’écrans
La page d’accueil et la zone d’écriture :

<img width="1920" height="1080" alt="homepage" src="https://github.com/user-attachments/assets/d77c065d-a02f-40f5-b29f-fe45465af018" />

L’éditeur avec une page en train d’être écrite :

<img width="1920" height="1080" alt="editor" src="https://github.com/user-attachments/assets/7546a84f-b172-4c0d-a20a-df6c2defbf3c" />

Une page publiée avec une image :

<img width="1920" height="1080" alt="published-page2" src="https://github.com/user-attachments/assets/0fc38a43-8bcc-4fbf-9087-ea4100be3e6c" />

## Copies du code source
- Radicle: [`rad:z2irFqQ3Zx1S8kqsVgH9itBoVUxkT`](https://radicle.network/nodes/iris.radicle.network/rad%3Az2irFqQ3Zx1S8kqsVgH9itBoVUxkT)
- Forgejo: [git.anarchists.space](https://git.anarchists.space/nonograph/nonograph)
- Archive: [Software Heritage Archive](https://archive.softwareheritage.org/browse/origin/directory/?origin_url=https://github.com/du82/nonograph&visit_type=git)

## Nom et origines
`anonymous` + `monograph` + `telegraph` = `nonograph`

Nonograph est une alternative open source à Telegraph (telegra.ph) orientée sur la vie privée et l’anonymat. Voici un [historique plutôt bien fait du projet](https://nonogra.ph/history-of-this-10-29-2026) que quelqu’un a fait dans le but de révéler mon identité.

« Nonograph est prononcé « none » + « oh » + « graphe ». None comme dans « aNONyme ».

## Audits et sécurité

- 10/13/2026 - [@du82](https://github.com/du82) a ajouté 13 200 dollars de Monero au programme de bug-bounty.
- 9/5/2026 - [@netqo](https://github.com/netqo) a contribué à des améliorations de sécurité. Il a été payé en Monero dans [cette pull request](https://github.com/du82/nonograph/pull/28).
* 8/21/2026 - [@sgpinkus](https://github.com/sgpinkus) a contribué à des améliorations de sécurité. Il a été payé en Monero dans [cette pull request](https://github.com/du82/nonograph/pull/26).
* 6/22/2026 - [@SmokeCamel](t.me/cigssss) a été remboursé pour nous avoir fourni des machines virtuelles isolées dans le but de faire tourner les meilleurs modèles d’IA contre le parser de Nonogram. Aucune vulnérabilité n’a été trouvée.
* 4/20/2026 - [@h_2_o0](https://t.me/h_2_o0) a trouvé une vulnérabilité servant à contourner la validation des URL le 4/20 (W), réparée dans [ce commit](https://github.com/du82/nonograph/commit/639f64f010e2b287bf3429af1814dd4fb8697a16).
* 10/15/2025 - [Security Assessment Report Redacted.pdf](https://github.com/user-attachments/files/27242849/Security.Assessment.Report.Redacted.pdf) - audit de la version initiale (v0.0.1), payé en Monero. Seuls les noms des auditeurs et leurs addresses e-mail ont été censurés. Réparé dans [ce commit](https://github.com/du82/nonograph/commit/2641fcaed1aaf458e69217e5489a75c93446b0d2) et [ce commit](https://github.com/du82/nonograph/commit/98178a380324270da704aa80e035aea012e6e748).

## Licence
Domaine public ([Unlicense](https://unlicense.org)). Ce logiciel appartient à tout le monde. Utilisez-le, modifiez-le, partagez-le sans aucune restriction. Faites-en ce que vous voulez, mais n’oubliez pas qu’il n’y a aucune garantie. Veuillez respecter les règles des serveurs Nonograph hébergés par la communauté que vous trouvez.