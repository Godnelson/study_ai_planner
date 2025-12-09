########################################
# Stage 1 — Build do binário Rust
########################################
FROM rust:1.85 AS builder

WORKDIR /build

# Copiamos apenas os manifestos primeiro para maximizar cache
COPY Cargo.toml Cargo.lock* ./

# Preparamos estrutura mínima para compilar dependências
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release || true

# Agora sim, copiamos o código real
COPY src ./src
COPY static ./static

# Compila o binário release definitivo
RUN cargo build --release


########################################
# Stage 2 — Imagem final minimalista
########################################
FROM debian:bookworm-slim AS runtime

ENV PORT=3000

EXPOSE 3000

# Instala apenas os certificados necessários
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copia o binário e os assets
WORKDIR /app
COPY --from=builder /build/target/release/study_ai_planner /app/study_ai_planner
COPY --from=builder /build/static /app/static

ENTRYPOINT ["/app/study_ai_planner"]
