# Aşama 1: Build
FROM rust:latest AS builder

WORKDIR /app

# Bağımlılıkları önce kopyala (cache optimizasyonu)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Gerçek kaynak kodunu kopyala ve derle
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# Aşama 2: Runtime
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/koop-api ./koop-api
COPY --from=builder /app/migrations ./migrations

EXPOSE 3000
CMD ["./koop-api"]
