# syntax=docker/dockerfile:1.7
# Bullpen in one RunPod GPU pod: the chat model, the store, the engine and the bot. The store
# needs a Postgres outside the pod. RunPod pods cannot run containers, so supervisord runs every service as
# a process and run_sandbox stays off. The model cache lives under /workspace, the pod volume.
ARG LLAMA_IMAGE=ghcr.io/ggml-org/llama.cpp:server-cuda

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

# llama.cpp server image: Ubuntu 24.04 with the CUDA runtime and /app/llama-server.
FROM ${LLAMA_IMAGE}
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl supervisor \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/engine /usr/local/bin/engine
COPY --from=builder /app/target/release/store /usr/local/bin/store
COPY --from=builder /app/target/release/discord /usr/local/bin/discord
COPY bullpen.toml /etc/bullpen/bullpen.toml
COPY deploy/runpod/supervisord.conf /etc/bullpen/supervisord.conf
COPY deploy/runpod/start.sh /usr/local/bin/bullpen-start

# Addresses inside the pod. Secrets come from the pod environment.
# llama-server links its shared libraries from /app.
ENV LD_LIBRARY_PATH=/app:${LD_LIBRARY_PATH}
ENV BULLPEN_CONFIG_FILE=/etc/bullpen/bullpen.toml \
    BULLPEN_DATA_DIR=/workspace \
    LLAMA_CACHE=/workspace/models \
    BULLPEN_APP__HTTP_ADDR=127.0.0.1:8080 \
    BULLPEN_ENGINE__BASE_URL=http://127.0.0.1:8080 \
    BULLPEN_STORE__BIND=127.0.0.1:8090 \
    BULLPEN_STORE__URL=http://127.0.0.1:8090 \
    BULLPEN_MODEL__BASE_URL=http://127.0.0.1:8000/v1 \
    BULLPEN_MODEL__API_KEY=local \
    BULLPEN_SANDBOX__ENABLED=false \
    BULLPEN_SANDBOX__REQUIRED=false \
    BULLPEN_CHAT_GGUF=unsloth/Qwen3.5-9B-GGUF:Q4_K_M \
    BULLPEN_CHAT_CTX=16384 \
    BULLPEN_CHAT_PARALLEL=2 \
    BULLPEN_MODEL__NAME=unsloth/Qwen3.5-9B-GGUF:Q4_K_M
WORKDIR /
ENTRYPOINT ["/usr/local/bin/bullpen-start"]
