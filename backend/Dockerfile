FROM rust:1.96-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

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
