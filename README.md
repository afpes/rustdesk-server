# Servidor do app-remoto (fork do rustdesk-server)

Este fork é o servidor RustDesk (hbbs/hbbr) usado pelo **app-remoto**. Ninguém instala nada a
partir daqui: cada push na branch `remoto` roda os testes e publica a imagem
`ghcr.io/<dono do repositório>/remoto-servidor` (pública), que o `docker-compose.yml` do `app-remoto`
baixa. Instalação completa: `DEPLOY.md` no repositório `app-remoto`.

## O que muda em relação ao oficial

Tudo fica em `src/remoto.rs` (mais poucas linhas em `src/rendezvous_server.rs`, marcadas com `app-remoto:`):

- **Troca de chaves (TCP seguro).** O cliente RustDesk logado numa API exige que o servidor faça a troca
  de chaves na porta 21116; o servidor oficial gratuito não faz e o cliente falha com
  *"Failed to secure tcp: deadline has elapsed"*.
- **Login obrigatório.** Com `MUST_LOGIN=Y`, o pedido de conexão só passa com um token válido do
  Console (JWT HS256 assinado pela API do app-remoto). Sem token, o técnico vê
  "Faça login no Console para conectar".

## Variáveis de ambiente (além das do oficial)

| Variável | O que é |
|---|---|
| `MUST_LOGIN` | `Y` = exige login no Console para conectar (padrão `N`) |
| `RUSTDESK_API_JWT_KEY` | segredo que valida o token; o mesmo `JWT_KEY` da API (o compose já passa) |

## Versões e atualização

| Ação | Como |
|---|---|
| Publicar uma versão nova | push na branch `remoto` (ou *Actions › Imagem do servidor › Run workflow*); a imagem sai com as tags `latest` e o sha do commit |
| Usar no app-remoto | `RUSTDESK_SERVER_TAG=<sha>` no `.env` do servidor e `sudo ./install.sh` |
| Trazer o oficial | `git fetch upstream && git merge upstream/master` na branch `remoto` |
| Build local | `docker build -f Dockerfile.remoto -t remoto-servidor .` (roda `cargo test --lib remoto` antes) |

---

*O restante desta página é o README original do rustdesk-server.*

# RustDesk Server Program

[![build](https://github.com/rustdesk/rustdesk-server/actions/workflows/build.yaml/badge.svg)](https://github.com/rustdesk/rustdesk-server/actions/workflows/build.yaml)

[**Download**](https://github.com/rustdesk/rustdesk-server/releases)

[**Manual**](https://rustdesk.com/docs/en/self-host/)

[**Configuration & environment variables**](docs/environment-variables.md)

[**FAQ**](https://github.com/rustdesk/rustdesk/wiki/FAQ)

[**How to migrate OSS to Pro**](https://rustdesk.com/docs/en/self-host/rustdesk-server-pro/installscript/#convert-from-open-source)

Self-host your own RustDesk server, it is free and open source.

> [!IMPORTANT]
> **Need more features?** [RustDesk Server Pro](https://rustdesk.com/pricing.html) might suit you better.
>
> **Want to develop your own server?** Start with [rustdesk-server-demo](https://github.com/rustdesk/rustdesk-server-demo), a simpler starting point than this repository.

## How to build manually

```bash
cargo build --release
```

Three executables will be generated in target/release.

- hbbs - RustDesk ID/Rendezvous server
- hbbr - RustDesk relay server
- rustdesk-utils - RustDesk CLI utilities

You can find updated binaries on the [Releases](https://github.com/rustdesk/rustdesk-server/releases) page.

## Configuration

`hbbs` and `hbbr` can be configured with command-line flags, environment
variables, or an `.env` / config file. Run `hbbs --help` or `hbbr --help` to see
the available flags.

The most common options:

| Option | Flag | Env var | Applies to | Purpose |
| --- | --- | --- | --- | --- |
| Key | `-k` | `KEY` | hbbs, hbbr | `hbbs` loads/generates one by default |
| Bind address | `-b` | `BIND` | hbbs, hbbr | Local IP address to listen on (default: all interfaces; requires 1.1.17+) |
| Port | `-p` | `PORT` | hbbs, hbbr | Listening port (hbbs `21116`, hbbr `21117`) |
| Relay servers | `-r` | `RELAY-SERVERS` | hbbs | Override when the relay uses a different address or a non-standard port |
| Force relay | — | `ALWAYS_USE_RELAY` | hbbs | `Y` disables direct connections |
| Log level | — | `RUST_LOG` | hbbs, hbbr | e.g. `debug` (default `info`) |

See **[docs/environment-variables.md](docs/environment-variables.md)** for the
full list of variables, the file/flag/env precedence rules, database and relay
bandwidth tuning, Docker image variables, and examples.

## Installation

Please follow this [doc](https://rustdesk.com/docs/en/self-host/rustdesk-server-oss/)
