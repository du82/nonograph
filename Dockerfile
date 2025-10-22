FROM rust:1.82-slim-bullseye as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

COPY src/ ./src/
COPY templates/ ./templates/
COPY Config.toml ./
RUN touch src/main.rs && cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    tor \
    sudo \
    && rm -rf /var/lib/apt/lists/*

RUN echo "DataDirectory /var/lib/tor" > /etc/tor/torrc && \
    echo "SocksPort 0" >> /etc/tor/torrc && \
    echo "ControlSocket 0" >> /etc/tor/torrc && \
    echo "" >> /etc/tor/torrc && \
    echo "HiddenServiceDir /var/lib/tor/hidden_service/" >> /etc/tor/torrc && \
    echo "HiddenServicePort 80 127.0.0.1:8009" >> /etc/tor/torrc



RUN useradd -r -s /bin/false nonograph
RUN mkdir -p /app/content /app/templates /var/lib/tor && \
    mkdir -p /var/lib/tor/hidden_service && \
    chmod 700 /var/lib/tor && \
    chmod 700 /var/lib/tor/hidden_service && \
    chown -R nonograph:nonograph /app && \
    chown -R debian-tor:debian-tor /var/lib/tor && \
    echo "nonograph ALL=(debian-tor) NOPASSWD: /usr/bin/tor" >> /etc/sudoers

COPY --from=builder /app/target/release/nonograph /app/
COPY --from=builder /app/Config.toml /app/
COPY --from=builder /app/templates/ /app/templates/

RUN sed -i 's/address = "127.0.0.1"/address = "0.0.0.0"/' /app/Config.toml || true

USER nonograph
WORKDIR /app
EXPOSE 8009

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8009

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8009/ || exit 1

CMD ["sh", "-c", "sudo -u debian-tor tor -f /etc/tor/torrc & exec ./nonograph"]
