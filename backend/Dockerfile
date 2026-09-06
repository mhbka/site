FROM rust:1.96-bookworm AS chef

RUN cargo install --locked --version 0.1.78 cargo-chef

WORKDIR /app

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release --locked

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --create-home app

COPY --from=builder /app/target/release/blog-backend /usr/local/bin/blog-backend

USER app
EXPOSE 8080

CMD ["blog-backend"]
