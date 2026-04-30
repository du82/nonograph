<div align="center">
  <img align="center" width="96" height="96" alt="android-chrome-512x512" src="https://github.com/user-attachments/assets/9a06a3fe-46ee-422c-93ad-ce3e504603c0" />
</div>

<h1 align="center">Nonograph</h1>

<p align="center"><b>为注重隐私的网络提供匿名发布服务</b></p>
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

[English](README.md) | 简体中文

</div>

用于自建匿名发布平台。无需账号，没有跟踪。用于撰写、发布、分享。不收集其他任何信息。 

https://github.com/user-attachments/assets/d662c9a2-f0ed-4266-bf55-e2c1f024269e

## 已知实例

| 运行状态 | 位置 | 公网域名 | 洋葱网络 |
|--------|----------|----------|-------|
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnonogra.ph) | 🏴‍☠️ Unknown | https://nonogra.ph | http://aue5jcgehi2uq5gdrxuhfqmyw4yfrsq3ggd7bvcydqyhlnwha27iqiad.onion/ |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fwrite.daun.world) | 🏴‍☠️ Unknown | https://write.daun.world/ | http://fmoigm7j3z6vh4hgssdfhlt6knkp443thgxpe5wmbaevvb5km2d3suyd.onion/ |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fproxy.write.daun.world) | 🇫🇮 Finland | https://proxy.write.daun.world/ | 见上文 |
| ![Website](https://img.shields.io/website?url=https%3A%2F%2Fnull) | 🇰🇿 Kazakhstan | | http://5mq3db45agipsceghnpx3iumlctya3absmp4sgnitqcmrmhaqhbbjcid.onion/ |

## 部署

```bash
git clone https://github.com/du82/nonograph
cd nonograph
make up
```

若有需要，`make up` 会安装 Docker，构建包含 Tor 的容器，并打印出你的 `.onion` 地址。 

```bash
make up        # 在 Tor 上启动服务
make tor       # 打印 .onion 地址
make status    # 检查状态
make down      # 停止容器
make clean     # 彻底移除容器
```
讨厌 Docker 吗？运行 `./run` 以原生方式构建并运行（仅限 Linux）。

## 功能
- 支持带表格、代码块、脚注和 `#spoiler#` 语法的 Markdown 格式
- 可从 URL 嵌入图片和视频
- 无需账户，不记录 IP 地址，不进行数据分析
- 开箱即用的 Tor 隐藏服务
- 仅需 64MB 内存即可运行。在树莓派或便宜的 VPS 上也能良好运行 

## 依赖环境
- 基于 Debian 的 Linux 系统（如 Raspberry Pi OS、KDE Neon、Pop_OS 等，不包括 Ubuntu）
- 64MB 内存，64MB 磁盘空间

## 项目名称灵感
`anonymous`（匿名） + `monograph`（专题著作） + `telegraph`（电报） = `nonograph`

## 安全审计
- [《安全评估报告（已编辑）.pdf》](https://github.com/user-attachments/files/27242849/Security.Assessment.Report.Redacted.pdf) - 2025 年 10 月 15 日对初始版本（v0.0.1）的审计，费用以门罗币支付。仅审计员姓名和电子邮件被编辑。


## 许可协议
公共领域（[Unlicense](https://unlicense.org)）。该软件属于所有人。无限制使用、修改或分享，无需署名，不附带任何条件，不提供任何保证。
