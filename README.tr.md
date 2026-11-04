<div align="center">
  <img align="center" width="96" height="96" alt="android-chrome-512x512" src="https://github.com/user-attachments/assets/9a06a3fe-46ee-422c-93ad-ce3e504603c0" />
</div>

<h1 align="center">Nonograph</h1>

<p align="center"><b>Gizliliğe önem verenler için anonim yayın platformu.</b></p>
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

[English](README.md) | [简体中文](README.zh.md) | [Русский](README.ru.md) | [Español](README.es.md) | [Français](README.fr.md) | [Magyar](README.hu.md) | Türkçe

</div>

Nonograph, yazılarını kimliğini açığa çıkarmadan internette yayımlamak isteyen herkes için tasarlanmış, sade bir anonim yayın platformudur. Yazınızı yazar, yayımlar ve size verilen bağlantıyı paylaşırsınız. Hesap açmanız veya profil oluşturmanız gerekmez; okuyucularınız da herhangi bir analiz aracıyla takip edilmez.

https://github.com/user-attachments/assets/d662c9a2-f0ed-4266-bf55-e2c1f024269e

## Bilinen Sunucular

Aşağıdaki sunucular Nonograph'ı herkesin kullanımına açmaktadır. Sunuculara clearnet adresinden veya Tor Browser ile erişilebilen onion adresinden ulaşabilirsiniz.

| Uptime                                                                               | Konum         | Clearnet                         | Onion                                                                  |   |
|--------------------------------------------------------------------------------------|---------------|----------------------------------|------------------------------------------------------------------------|---|
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonogra.ph)              | 🏴‍☠️ Bilinmiyor  | https://nonogra.ph               | http://ortmy3ey5usdzf4ivht6axtb72owjniaeqrexknosyons544aooltzyd.onion/ |   |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fwrite.eversiege.network) | 🏴‍☠️ Bilinmiyor  | https://write.eversiege.network/ | http://fmoigm7j3z6vh4hgssdfhlt6knkp443thgxpe5wmbaevvb5km2d3suyd.onion/ |   |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonograph.com)           | 🇭🇰 Hong Kong  | https://nonograph.com<br>https://nonograph.ink            | http://gt65bmujun7alps7b7oar5x2u5lprxjdvpwrxcrnwkgheocjwllchhqd.onion/ |   |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fproxy.write.daun.world)  | 🇫🇮 Finlandiya | https://proxy.write.daun.world/  | Yukarıya bakınız                                                       |   |
| Onion                                                                                | 🇷🇺 Rusya      |                                  | http://q2w7sdjlmfc5vjif6y3372665wzyqskqvjji7bmhfn72orxcbljvonid.onion/ |   |
| Onion                                                                                | 🇭🇺 Macaristan |                                  | http://oahjeuod7h7kiu7nfhasx34djvnwtlgzogk7a5t7kl2gqskskhi4r5qd.onion/ |   |
| Onion                                                                                | 🏴‍☠️ Bilinmiyor  |                                  | http://uawaa47jvsfr3ij63ns25xp6qvhqswsx3fgij2evbrcnt3ygxq3dbwyd.onion/ |   |
| Onion                                                                                | 🇰🇿 Kazakistan |                                  | http://5mq3db45agipsceghnpx3iumlctya3absmp4sgnitqcmrmhaqhbbjcid.onion/ |   |

Bu sunucular, her biri kendi kullanım koşullarına sahip bağımsız kişi ve kuruluşlar tarafından işletilmektedir. Size en uygun olanı seçebilir veya Nonograph'ı self-host edebilirsiniz.

## Kurulum

Nonograph'ı Docker ile tek komutla kurabilirsiniz:

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

Dilerseniz kaynak kodunu indirip container'ı kendiniz de build edebilirsiniz:

```bash
git clone https://github.com/du82/nonograph
cd nonograph
make up
```

Kurulum tamamlandıktan sonra, sunucunuzun `.onion` adresini aşağıdaki komutla log'larda görebilirsiniz:

```bash
docker logs nonograph
```

Docker kullanmak istemiyorsanız, `./run` komutuyla Nonograph'ı native olarak build edip çalıştırabilirsiniz (yalnızca Debian için).

## Özellikler

Nonograph, yazılarınızı biçimlendirmek için çok sayıda seçenek sunar. Kullanılabilir seçeneklerin listesini görmek için yeni bir satırın başına `/` yazmanız yeterlidir.

<img width="561" height="447" alt="image" src="https://github.com/user-attachments/assets/cda96a9c-08bc-4add-bf5b-e8eb0b352201" />


## Ekran Görüntüleri

Ana sayfa ve yazı alanı:

<img width="1920" height="1080" alt="homepage" src="https://github.com/user-attachments/assets/d77c065d-a02f-40f5-b29f-fe45465af018" />

Yazım sürecindeki bir sayfa:

<img width="1920" height="1080" alt="editor" src="https://github.com/user-attachments/assets/7546a84f-b172-4c0d-a20a-df6c2defbf3c" />

Görsel içeren, yayımlanmış bir sayfa:

<img width="1920" height="1080" alt="published-page2" src="https://github.com/user-attachments/assets/0fc38a43-8bcc-4fbf-9087-ea4100be3e6c" />

## Diğer Platformlar

Proje, aşağıdaki platformda da yayımlanmaktadır:

- Radicle: [`rad:z2irFqQ3Zx1S8kqsVgH9itBoVUxkT`](https://radicle.network/nodes/iris.radicle.network/rad%3Az2irFqQ3Zx1S8kqsVgH9itBoVUxkT)

## İsmin Kökeni

`anonymous` (anonim) + `monograph` (monografi) + `telegraph` (telgraf) = `nonograph`

Nonograph, gizlilik ve anonimliği ön planda tutan, açık kaynaklı bir Telegraph (telegra.ph) alternatifidir. Projenin geçmişini ayrıntılı biçimde anlatan bir [tarihçe](https://nonogra.ph/history-of-this-10-29-2026) mevcuttur; bu metin, geliştiricinin kimliğini ortaya çıkarmak amacıyla üçüncü bir kişi tarafından hazırlanmıştır.

"Nonograph" kelimesi "non-o-graf" şeklinde okunur. Baştaki "non", İngilizce "anonymous" (anonim) kelimesinden gelir.

## Güvenlik ve Denetimler

Nonograph'ın güvenliği, bağımsız araştırmacılar tarafından düzenli olarak incelenmekte ve bulunan açıklar bug bounty programı kapsamında ödüllendirilmektedir.

- **13 Ekim 2026:** [@du82](https://github.com/du82), bug bounty programına 13.200 ABD doları değerinde Monero ekledi.
- **5 Eylül 2026:** [@netqo](https://github.com/netqo), güvenlik iyileştirmelerine katkıda bulundu. Ödeme, [bu pull request](https://github.com/du82/nonograph/pull/28) üzerinden Monero ile yapıldı.
- **21 Ağustos 2026:** [@sgpinkus](https://github.com/sgpinkus), güvenlik iyileştirmelerine katkıda bulundu. Ödeme, [bu pull request](https://github.com/du82/nonograph/pull/26) üzerinden Monero ile yapıldı.
- **22 Haziran 2026:** [@SmokeCamel](t.me/cigssss), Nonograph'ın parser'ını en gelişmiş yapay zekâ modelleriyle test etmek için izole VM'ler sağladı; masrafları karşılandı. Herhangi bir güvenlik açığı bulunmadı.
- **20 Nisan 2026:** [@h_2_o0](https://t.me/h_2_o0), bir URL validation bypass açığı tespit etti. Açık, [bu commit](https://github.com/du82/nonograph/commit/639f64f010e2b287bf3429af1814dd4fb8697a16) ile giderildi.
- **15 Ekim 2025:** İlk sürüm (v0.0.1) bağımsız bir güvenlik denetiminden (audit) geçirildi ve ücreti Monero ile ödendi. [Denetim raporuna buradan ulaşabilirsiniz](https://github.com/user-attachments/files/27242849/Security.Assessment.Report.Redacted.pdf); raporda yalnızca denetçinin adı ve e-posta adresi gizlenmiştir. Tespit edilen sorunlar [bu](https://github.com/du82/nonograph/commit/2641fcaed1aaf458e69217e5489a75c93446b0d2) ve [bu](https://github.com/du82/nonograph/commit/98178a380324270da704aa80e035aea012e6e748) commit'lerle giderildi.

## Lisans

Nonograph kamu malıdır ([Unlicense](https://unlicense.org)) ve herkese aittir. Yazılımı hiçbir kısıtlama olmaksızın kullanabilir, değiştirebilir ve paylaşabilirsiniz. Ancak yazılım herhangi bir garanti olmaksızın sunulmaktadır. Nonograph'ı barındıran sunucuları kullanırken lütfen o sunucuların kullanım koşullarına uyunuz.

## Teşekkürler

Nonograph; Rusça, Macarca, İspanyolca, Portekizce ve Ukraynacaya gönüllü çevirmenler tarafından çevrilmiştir. Projenin güvenliği, kodu titizlikle inceleyen, beklenmedik hataları ortaya çıkaran ve bu katkıları karşılığında ödüllendirilen kişiler sayesinde sağlanmaktadır. Dünyanın dört bir yanında onlarca sunucu, Nonograph'ı herkesin ücretsiz kullanımına sunmaktadır. Logo ise emeğinin karşılığını alan bir sanatçı tarafından tasarlanmıştır. Bu projeye katkıda bulunan herkese içtenlikle teşekkür ederiz.
