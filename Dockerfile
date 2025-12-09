# ============================
# 1ª etapa: build do binário
# ============================
FROM rust:1.85 as builder

WORKDIR /app

# Copia manifestos primeiro (melhora cache)
COPY Cargo.toml Cargo.lock* ./

# Cria um dummy src/lib.rs pra poder compilar dependências em cache
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# Agora copia o código real
COPY src ./src
COPY static ./static

# Build release de verdade
RUN cargo build --release

# ============================
# 2ª etapa: imagem final, enxuta
# ============================
FROM debian:bookworm-slim

# Certificados pra HTTPS (reqwest)
RUN apt-get update && apt-get install -y \
    ca-certificates \
  && update-ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copia o binário do builder
COPY --from=builder /app/target/release/study_ai_planner /app/study_ai_planner
# Copia os assets estáticos (frontend)
COPY --from=builder /app/static /app/static

# Render vai setar PORT, mas deixamos default pra rodar local se quiser
ENV PORT=3000

EXPOSE 3000

CMD ["./study_ai_planner"]
