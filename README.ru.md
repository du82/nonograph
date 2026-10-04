<div align="center">
  <img align="center" width="96" height="96" alt="android-chrome-512x512" src="https://github.com/user-attachments/assets/9a06a3fe-46ee-422c-93ad-ce3e504603c0" />
</div>

<h1 align="center">Nonograph</h1>

<p align="center"><b>Анонимная публикация записей для тех, кто ценит конфиденциальность в интернете.</b></p>
<div align="center">
  <a href="https://unlicense.org">
    <img alt="Лицензия" src="https://img.shields.io/github/license/du82/nonograph">
  </a>
  <a href="https://github.com/du82/nonograph/releases/latest">
    <img alt="Релизы" src="https://img.shields.io/github/v/release/du82/nonograph">
  </a>
  <a href="https://github.com/du82/nonograph/commits/main/">
    <img alt="GitHub commit activity" src="https://img.shields.io/github/commit-activity/m/du82/nonograph">
  </a>
  <a href="http://ortmy3ey5usdzf4ivht6axtb72owjniaeqrexknosyons544aooltzyd.onion/">
    <img src="https://img.shields.io/badge/Tor-Hidden%20Service-7d4698?style=flat&logo=torproject&logoColor=white" alt="Tor Hidden Service">
  </a>

[English](README.md) | [简体中文](README.zh.md) | Русский

</div>

Nonograph это простая платформа для анонимной публикации, созданная для тех, кто хочет разместить свои тексты в интернете, не раскрывая свою личность. Вы пишете текст, получаете ссылку, которой можно поделиться, и всё. Никакой регистрации, никаких профилей и никакой аналитики, отслеживающей действия читателей.
Nonograph является открытой альтернативой telegra.ph

https://github.com/user-attachments/assets/d662c9a2-f0ed-4266-bf55-e2c1f024269e

## Известные инстансы

| Аптайм                                                                               | Локация      | Clearnet                         | Onion                                                                  |   |
|--------------------------------------------------------------------------------------|---------------|----------------------------------|------------------------------------------------------------------------|---|
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonogra.ph)              | 🏴‍☠️ Неизвестно    | https://nonogra.ph               | http://ortmy3ey5usdzf4ivht6axtb72owjniaeqrexknosyons544aooltzyd.onion/ |   |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fwrite.eversiege.network) | 🏴‍☠️ Неизвестно    | https://write.eversiege.network/ | http://fmoigm7j3z6vh4hgssdfhlt6knkp443thgxpe5wmbaevvb5km2d3suyd.onion/ |   |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonograph.com)           | 🇭🇰 Гонконг  | https://nonograph.com            | http://gt65bmujun7alps7b7oar5x2u5lprxjdvpwrxcrnwkgheocjwllchhqd.onion/ |   |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fproxy.write.daun.world)  | 🇫🇮 Финляндия    | https://proxy.write.daun.world/  | see above                                                              |   |
| Onion                                                                                | 🇷🇺 Россия     |                                  | http://q2w7sdjlmfc5vjif6y3372665wzyqskqvjji7bmhfn72orxcbljvonid.onion/ |   |
| Onion                                                                                | 🇭🇺 Венгрия    |                                  | http://t7fgh7qvjysh3wer747m6dkjvkjsqvajyv5bh2grzjgpd2derxsxbdad.onion/ |   |
| Onion                                                                                | 🏴‍☠️ Неизвестно    |                                  | http://uawaa47jvsfr3ij63ns25xp6qvhqswsx3fgij2evbrcnt3ygxq3dbwyd.onion/ |   |
| Onion                                                                                | 🇰🇿 Казахстан |                                  | http://5mq3db45agipsceghnpx3iumlctya3absmp4sgnitqcmrmhaqhbbjcid.onion/ |   |

Эти инстансы предоставлены третьими сторонами, каждая со своими правилами. Выберите вариант, подходящий вам больше всего, или разместите сайт самостоятельно.

## Развёртывание

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

или возьмите исходный код и сделайте свой контейнер:

```bash
git clone https://github.com/du82/nonograph
cd nonograph
make up
```

затем проверьте логи вашего `.onion` адреса:

```bash
docker logs nonograph
```

Ненавидите докер? Запустите `./run`, чтобы собрать и запустить нативно (только Debian).

## Функции
Nonograph поставляется с обширным списком markdown форматирования; введите `/` на новой строке, чтобы вывести список вариантов.

<img width="561" height="447" alt="image" src="https://github.com/user-attachments/assets/cda96a9c-08bc-4add-bf5b-e8eb0b352201" />


## Скриншоты
Главная страница и поле ввода:

<img width="1920" height="1080" alt="homepage" src="https://github.com/user-attachments/assets/d77c065d-a02f-40f5-b29f-fe45465af018" />

Редактор в процессе написания страницы:

<img width="1920" height="1080" alt="editor" src="https://github.com/user-attachments/assets/7546a84f-b172-4c0d-a20a-df6c2defbf3c" />

Опубликованная страница с изображением:

<img width="1920" height="1080" alt="published-page2" src="https://github.com/user-attachments/assets/0fc38a43-8bcc-4fbf-9087-ea4100be3e6c" />

## Также доступен на...
- Radicle: [`rad:z2irFqQ3Zx1S8kqsVgH9itBoVUxkT`](https://radicle.network/nodes/iris.radicle.network/rad%3Az2irFqQ3Zx1S8kqsVgH9itBoVUxkT)

## Происхождение названия
`anonymous` + `monograph` + `telegraph` = `nonograph`

## Аудиты и безопасность

* 05.09.2026 - [@netqo](https://github.com/netqo) внёс улучшения безопасности. Вознаграждение выплачено в Monero в [этом pull request](https://github.com/du82/nonograph/pull/28).
* 21.08.2026 - [@sgpinkus](https://github.com/sgpinkus) внёс улучшения безопасности. Вознаграждение выплачено в Monero в [этом pull request](https://github.com/du82/nonograph/pull/26).
* 22.06.2026 - [@SmokeCamel](t.me/cigssss) получил компенсацию затрат и предоставил изолированные виртуальные машины для проверки парсера Nonograph с помощью ведущих ИИ-моделей. Уязвимостей не обнаружено.
* 20.04.2026 - [@h_2_o0](https://t.me/h_2_o0) обнаружил обход валидации URL 4/20 (отлично), исправлено [в этом коммите](https://github.com/du82/nonograph/commit/639f64f010e2b287bf3429af1814dd4fb8697a16).
* 15.10.2025 - [Security Assessment Report Redacted.pdf](https://github.com/user-attachments/files/27242849/Security.Assessment.Report.Redacted.pdf) - Аудит первой версии (v0.0.1), оплаченный в Monero. Скрыты только имя и email аудитора. Исправлено в [этом](https://github.com/du82/nonograph/commit/2641fcaed1aaf458e69217e5489a75c93446b0d2) и [этом](https://github.com/du82/nonograph/commit/98178a380324270da704aa80e035aea012e6e748) коммитах.

## Лицензия
Публичный домен ([Unlicense](https://unlicense.org)). Это программное обеспечение принадлежит всем. Используйте его, изменяйте и распространяйте без ограничений. Указание авторства не требуется, никаких дополнительных условий. Гарантии не предоставляются.
