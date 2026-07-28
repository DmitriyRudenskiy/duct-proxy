# mitmproxy-rs

MITM-прокси на Rust, вдохновлённый [mitmproxy](https://mitmproxy.org).

## Возможности
- HTTP/HTTPS проксирование (explicit mode)
- TLS interception с on-the-fly генерацией сертификатов
- Система аддонов (ModifyHeaders, Block, ModifyBody)
- Логирование трафика
- Конфигурация через YAML

## Быстрый старт

### Установка
```bash
cargo build --release
```

### Запуск
```bash
./target/release/mitm-cli --port 8080
```

### Использование
```bash
# HTTP
curl -x http://127.0.0.1:8080 http://example.com

# HTTPS (с доверием к CA)
curl -x http://127.0.0.1:8080 \
     --cacert ~/.mitmproxy/mitmproxy-ca-cert.pem \
     https://example.com
```

### Установка CA в систему
```bash
# Linux (Debian/Ubuntu)
sudo cp ~/.mitmproxy/mitmproxy-ca-cert.pem /usr/local/share/ca-certificates/
sudo update-ca-certificates

# macOS
sudo security add-trusted-cert -d -r trustRoot \
     -k /Library/Keychains/System.keychain \
     ~/.mitmproxy/mitmproxy-ca-cert.pem
```

## Архитектура
```
crates/
├── mitm-core      # Flow, Connection, Headers — базовые типы
├── mitm-net       # HTTP parsing, URL, cookies, chunked encoding
├── mitm-proxy     # Proxy server, TLS interception, forwarding
├── mitm-certs     # CA, CertStore, leaf certificates, SNI
├── mitm-addons    # Addon trait, AddonManager, built-in addons
├── mitm-options   # CLI args, config file, OptManager
├── mitm-io        # Flow serialization, dump format, HAR
└── mitm-cli       # Binary entry point
```

## Конфигурация
```yaml
# ~/.mitmproxy/config.yaml
listen_host: "0.0.0.0"
listen_port: 8080
mode: explicit
ssl_insecure: false
```

## Разработка
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

## Лицензия
MIT
