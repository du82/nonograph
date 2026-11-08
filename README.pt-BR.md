<div align="center">
  <img align="center" width="96" height="96" alt="android-chrome-512x512" src="https://github.com/user-attachments/assets/9a06a3fe-46ee-422c-93ad-ce3e504603c0" />
</div>

<h1 align="center">Nonograph</h1>

<p align="center"><b>Plataforma de publicação anônima na web que respeita a sua privacidade.</b></p>
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

English | [简体中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md) | Português (BR) | [Français](README.fr.md) | [Magyar](README.hu.md) | [Türkçe](README.tr.md) | [Português (BR)](README.pt-BR.md)

</div>

O Nonograph é uma plataforma de publicação simples e anônima para quem quer que suas palavras existam online sem que sejam atreladas a sua identidade. Você escreve, recebe um link de compartilhamento, e é isso. Sem necessidade de criação de conta, sem perfil ou estatísticas que rastreiam os leitores. 

https://github.com/user-attachments/assets/d662c9a2-f0ed-4266-bf55-e2c1f024269e

## Instâncias Conhecidas

|                                        Uptime                                        | Localização        | Clearnet                                    | Onion                                                                  |
| :----------------------------------------------------------------------------------: | :----------------- | :------------------------------------------ | :--------------------------------------------------------------------- |
|       ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonogra.ph)        | 🏴‍☠️ Desconhecida | https://nonogra.ph                          | http://ortmy3ey5usdzf4ivht6axtb72owjniaeqrexknosyons544aooltzyd.onion/ |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fwrite.eversiege.network) | 🏴‍☠️ Desconhecida | https://write.eversiege.network/            | http://fmoigm7j3z6vh4hgssdfhlt6knkp443thgxpe5wmbaevvb5km2d3suyd.onion/ |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fwrite.anarchists.space)  | 🏴‍☠️ Desconhecida | https://write.anarchists.space/             | http://qrb7g7gnowbpxxbpjeqaxr3a6odcvipd2xdumemvqj2nyfusghzzv6yd.onion  |
|      ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonograph.com)      | 🇭🇰 Hong Kong     | https://nonograph.com https://nonograph.ink | http://gt65bmujun7alps7b7oar5x2u5lprxjdvpwrxcrnwkgheocjwllchhqd.onion/ |
|       ![Website](https://img.shields.io/website?url=https%3A%2F%2Fdisc.us.ci)        | 🏴‍☠️ Desconhecida | https://disc.us.ci/                         | Tor inacessível                                                        |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fproxy.write.daun.world)  | 🇫🇮 Finlândia     | https://proxy.write.daun.world/             | http://fmoigm7j3z6vh4hgssdfhlt6knkp443thgxpe5wmbaevvb5km2d3suyd.onion/ |
|                                        Onion                                         | 🇷🇺 Rússia        |                                             | http://q2w7sdjlmfc5vjif6y3372665wzyqskqvjji7bmhfn72orxcbljvonid.onion/ |
|                                        Onion                                         | 🇭🇺 Hungria       |                                             | http://oahjeuod7h7kiu7nfhasx34djvnwtlgzogk7a5t7kl2gqskskhi4r5qd.onion/ |
|                                        Onion                                         | 🏴‍☠️ Desconhecida |                                             | http://uawaa47jvsfr3ij63ns25xp6qvhqswsx3fgij2evbrcnt3ygxq3dbwyd.onion/ |
|                                        Onion                                         | 🇰🇿 Cazaquistão   |                                             | http://5mq3db45agipsceghnpx3iumlctya3absmp4sgnitqcmrmhaqhbbjcid.onion/ |

Essas instâncias são fornecidas por terceiros, cada uma com suas próprias políticas. Escolha aquela que reflete melhor os seus valores, ou hospede a sua própria instância.

## Implementação

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

ou pegue o código-fonte e crie o seu próprio container:

```bash
git clone https://github.com/du82/nonograph
cd nonograph
make up
```

Então verifique os logs para o seu endereço `.onion`:

```bash
docker logs nonograph
```

Não gosta de Docker? Execute `./run` para compilar e rodar nativamente (apenas para Debian).

## Funcionalidades
Nonograph vem com uma lista extensa de opções de marcação; digite `/` em uma nova linha para exibi-las. 

<img width="561" height="447" alt="imagem" src="https://github.com/user-attachments/assets/cda96a9c-08bc-4add-bf5b-e8eb0b352201" />


## Capturas de tela
A página inicial e a área de escrita:

<img width="1920" height="1080" alt="página inicial" src="https://github.com/user-attachments/assets/d77c065d-a02f-40f5-b29f-fe45465af018" />

O editor com uma página em andamento:

<img width="1920" height="1080" alt="editor" src="https://github.com/user-attachments/assets/7546a84f-b172-4c0d-a20a-df6c2defbf3c" />

Uma página publicada com uma imagem:

<img width="1920" height="1080" alt="página publicada2" src="https://github.com/user-attachments/assets/0fc38a43-8bcc-4fbf-9087-ea4100be3e6c" />

## Código-fonte espelhado em...
- Radicle: [`rad:z2irFqQ3Zx1S8kqsVgH9itBoVUxkT`](https://radicle.network/nodes/iris.radicle.network/rad%3Az2irFqQ3Zx1S8kqsVgH9itBoVUxkT)
- Forgejo: [git.anarchists.space](https://git.anarchists.space/nonograph/nonograph)
- Arquivo: [Software Heritage Archive](https://archive.softwareheritage.org/browse/origin/directory/?origin_url=https://github.com/du82/nonograph&visit_type=git)

## Nome e origens
`anonymous` + `monograph` + `telegraph` = `nonograph`

Nonograph é uma alternativa de código aberto ao Telegraph (telegra.ph) com um foco em privacidade e anonimato. Aqui está um [histórico do projeto](https://nonogra.ph/history-of-this-10-29-2026) que alguém criou numa tentativa de me expor.

"Nonograph" é pronunciado como "non" + "oh" + "graph". Non, como em "aNONymous"

## Auditorias e segurança

- 13/10/2026 - [@du82](https://github.com/du82) adicionou o equivalente a 13.200 dólares americanos em Monero para o programa de bug bounty. 
- 05/09/2026 - [@netqo](https://github.com/netqo) contribuiu com melhorias de segurança. Pago em Monero [neste pull request](https://github.com/du82/nonograph/pull/28).
* 21/08/2026 - [@sgpinkus](https://github.com/sgpinkus) contribuiu com melhorias de segurança. Pago em Monero [neste pull request](https://github.com/du82/nonograph/pull/26).
* 22/06/2026 - [@SmokeCamel](t.me/cigssss) foi reembolsado e forneceu VMs isoladas para testar os principais modelos de IA contra o parser do Nonograph. Nenhuma vulnerabilidade foi encontrada.
* 20/04/2026 - [@h_2_o0](https://t.me/h_2_o0) encontrou desvio na validação de URL em 20/04 (belo timing), corrigido [neste commit](https://github.com/du82/nonograph/commit/639f64f010e2b287bf3429af1814dd4fb8697a16).
* 15/10/2025 - [Security Assessment Report Redacted.pdf](https://github.com/user-attachments/files/27242849/Security.Assessment.Report.Redacted.pdf) - auditoria da versão inicial (v0.0.1), paga em Monero. Apenas o nome e e-mail dos auditores foram omitidos. Corrigido [neste](https://github.com/du82/nonograph/commit/2641fcaed1aaf458e69217e5489a75c93446b0d2) e [neste](https://github.com/du82/nonograph/commit/98178a380324270da704aa80e035aea012e6e748) commit.

## Licença
Domínio público ([Unlicense](https://unlicense.org)). Este software pertence a todos. Use, modifique e compartilhe sem restrições. Faça o que quiser com ele, mas lembre-se de que não há garantia alguma. Por favor, respeite as políticas dos servidores Nonograph que você encontrar.
