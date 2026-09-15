# syntax=docker/dockerfile:1.7
# One image for the engine, the store and the discord bot; the entrypoint selects the binary.
FROM mirror.gcr.io/library/rust:1.95-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release -p engine -p store -p discord

# Runtime with the docker client that run_sandbox calls and curl for health checks.
FROM mirror.gcr.io/library/debian:bookworm-slim
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl docker.io \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/engine /engine
COPY --from=builder /app/target/release/store /store
COPY --from=builder /app/target/release/discord /discord
# Settings file; environment variables override it.
COPY bullpen.toml /bullpen.toml
EXPOSE 8080
ENTRYPOINT ["/engine"]
