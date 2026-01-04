# ============================
# Stage 1: build do binário
# ============================
FROM rust:1.85 as builder

WORKDIR /build

COPY Cargo.toml Cargo.lock* ./

RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release || true

COPY src ./src
COPY static ./static

RUN cargo build --release

# ============================
# Stage 2: runtime
# ============================
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

WORKDIR /app

COPY --from=builder /build/target/release/study_ai_planner /app/study_ai_planner
COPY --from=builder /build/static /app/static

RUN chmod +x /app/study_ai_planner

ENV PORT=3000
EXPOSE 3000

CMD ["/app/study_ai_planner"]
