# Deploy

## Local

```bash
just bootstrap     # .env, git hooks, deps
just engine        # then in separate shells: just discord, just web
just up            # the compose stack, built locally
```

The engine needs a market data key and the store: set `BULLPEN_MARKET__API_KEY` (free at https://finnhub.io/register), `BULLPEN_STORE__TOKEN`, and `BULLPEN_POSTGRES_PASSWORD` in `.env`, and `BULLPEN_MCP__HOSTED__URL` for a hosted MCP server. `just up` starts `postgres`, `store`, engine, discord and `sandboxd`; compose points the engine at `http://store:8090` and the store at `postgres`, and the store applies its migrations at boot. Postgres data lives in the `pgdata` volume. `engine`, `store` and `discord` share one image (`rust.Dockerfile`) with different entrypoints. `just model` starts the chat model (profile `model`).

## Production

```bash
git clone <your fork> bullpen && cd bullpen
cp .env.example .env               # secrets and per-machine URLs; BULLPEN_APP__ENV=production
# BULLPEN_IMAGE_OWNER is the GitHub user or organization CD pushes images to, in lower case.
BULLPEN_IMAGE_TAG=main just prod-up # pulls ghcr.io images
just prod-logs engine
```

`deploy/compose.prod.yml` overrides `compose.yml` with prebuilt images and no host ports except engine `:8080`. Phoenix, Prometheus, and Grafana are reachable only over a tunnel. Put a reverse proxy with TLS in front of the engine. Settings live in the committed `bullpen.toml`.

## RunPod

`deploy/docker/runpod.Dockerfile` builds `ghcr.io/<owner>/bullpen-runpod`: all of Bullpen in one GPU pod. A RunPod pod cannot start containers, so supervisord (`deploy/runpod/supervisord.conf`) runs the chat `llama-server`, the store, the engine, and the bot as processes. The store needs a Postgres outside the pod (a managed one, for example), and market data comes from Finnhub. It differs from compose in two ways:

- `run_sandbox` is off (`BULLPEN_SANDBOX__ENABLED=false`), so position math and scanned uploads are unavailable there.
- Phoenix is not included; export stays off until `BULLPEN_TELEMETRY__PHOENIX_URL` points at one.

Create the pod with the image, a volume at `/workspace` (the model cache), a host CUDA version of at least 12.8, and these environment variables: `BULLPEN_DISCORD__TOKEN`, `BULLPEN_DISCORD__GUILD_ID`, `BULLPEN_ENGINE__SERVICE_TOKEN`, `BULLPEN_MARKET__API_KEY`, `BULLPEN_STORE__TOKEN`, `BULLPEN_STORE__DATABASE_URL`, optionally `BULLPEN_MCP__HOSTED__URL`, `BULLPEN_APP__ENV=production`, and `HF_TOKEN` (a read token; community hosts share an IP that the Hugging Face Hub rate-limits without one). Every service listens on `127.0.0.1`; the pod exposes no port. The first boot downloads the GGUF into `/workspace/models`.

`BULLPEN_MODELS_API_KEY` makes the model server listen on every interface and require that key. Expose `8000/http` on the pod to let other apps use it. The engine uses the same key.

### Models only

With `BULLPEN_POD_ROLE=models` the pod runs only the model server, and `BULLPEN_MODELS_API_KEY` is required. Use it on a GPU pod when the engine and the bot run elsewhere. Point that engine at `https://<pod>-8000.proxy.runpod.net/v1` with the same key.

### Operating the pod

An always-on pod runs whether or not anyone talks to the bot, because the bot holds a Discord gateway connection. Billing stops only when the pod is stopped or terminated. A 4B chat model at a 16384 context fits in about 4 GB of VRAM.

| Task | How |
|---|---|
| Deploy a change | Merge to `main`; CD publishes `bullpen-runpod:<sha>` when its inputs change. Set the pod image to that tag. Changing the image restarts the pod. |
| Change a secret | Edit the pod environment in the RunPod console. The pod restarts with the new values. |
| Read logs | Pod logs in the console. Every service writes to the container log; supervisord prefixes its own state changes. |
| Restart | Restart the pod. `start.sh` reruns and supervisord starts every service. |
| Run a one-off command | Set the container start command to `bash -c "<command>; exec /usr/local/bin/bullpen-start"`, restart, read the output at the top of the logs, then clear the start command. The pod has no SSH key registered, so this is the way in. |
| Check the GPU | Run `nvidia-smi` as above. The console GPU memory reading is not reliable. |

State on the pod is the GGUF cache under `/workspace/models`; conversations and memory live in the Postgres the store points at. If the host fails, create a new pod as above; the first boot downloads the models again.

### Troubleshooting

Start with the pod logs. Search for `exited:`, `gave up` and `FATAL` to find the failing program; supervisord names it (`chat`, `engine`, `discord`), and the lines just before carry its own error.

| Symptom in the logs or Discord | Cause | Fix |
|---|---|---|
| Bot shows offline; `discord` exits at boot | Bad or rotated `BULLPEN_DISCORD__TOKEN`, or wrong `BULLPEN_DISCORD__GUILD_ID` | Correct the pod env. A healthy start logs `connected` with the guild id, then `commands registered`. |
| Bot is online but never answers | `chat` is not running | Find their exit reason below. The engine logs `could not read the chat server context` once at boot while the models load; that alone is harmless. |
| `chat` exits 127, `libllama-server-impl.so: cannot open shared object file` | `/app` is not on the library path | Add `LD_LIBRARY_PATH=/app` to the pod env; the image sets it. |
| `chat` logs `GET failed (429)` from the Hub | No `HF_TOKEN`; the community host shares a rate-limited IP | Set `HF_TOKEN` to a read token. Downloads land in `/workspace/models` and are reused after that. |
| `chat` logs `out of memory` or a CUDA allocation error | The model does not fit in VRAM | Lower `BULLPEN_CHAT_CTX`, pick a smaller GGUF, or move to a larger GPU. |
| Answers are very slow; `nvidia-smi` shows no `llama-server` processes | The models run on the CPU | Recreate the pod with a host CUDA of 12.8 or newer. |
| `engine` exits status 1 at boot | `Config::validate` rejected the configuration (a missing `BULLPEN_MARKET__API_KEY`, for example), or the store is unreachable | The line before the exit names the field or the failed store call. Fix the pod env or `bullpen.toml` and ship a new image. |
| Pod is gone, or the console says the machine is unavailable | The community host left | Create a new pod (see Operating the pod). |
| Pod stuck pulling the image, or `unauthorized` | The GHCR package is private, or the tag does not exist | Make `bullpen-runpod` public in the GitHub package settings; check the tag in the CD run. |
| CD `image (runpod)` job fails | A build input moved upstream (the llama.cpp server image, for example) | Read the failing step in the Actions log and pin or replace that input in `runpod.Dockerfile`. |

| Answers say market data rejected the key or hit its rate limit | A wrong `BULLPEN_MARKET__API_KEY`, or the free tier's per-minute limit | Fix the key; on the free tier, fewer parallel quotes or a paid plan. |

These lines appear on every healthy boot and need no action: the llama.cpp `control-looking token` warning and `phoenix url is unset`.

## Models

One `llama-server` container, chat on `:8000`. `just model` uses the CUDA image and reserves an NVIDIA device, so it needs a GPU and the container toolkit. Without one:

- `just model-cpu` applies `deploy/compose.cpu.yml`: the plain image, no device reservation, the same GGUF on the processor. Set `BULLPEN_CHAT_NGL=0`.
- Or point `BULLPEN_MODEL__BASE_URL` at any OpenAI-compatible endpoint, Baidu Qianfan's ERNIE models included, with its API key and `model.name`.

Details: `deploy/inference/README.md`.

## Web

`apps/web` builds to static files (`npm run build` to `apps/web/dist`). Deploy to Vercel (root directory `apps/web`) or any static host. It is not part of a Docker image.

## The sandbox

`run_sandbox` needs a container runtime. Compose provides `sandboxd`, a `docker:27-dind` daemon with no host port; the engine reaches it over `DOCKER_HOST=tcp://sandboxd:2375`, and the engine image carries only the client. On a developer host running `just engine`, the local `docker` serves.

Commands run in `bullpen-sandbox:main` (`deploy/docker/sandbox.Dockerfile`, built by `just sandbox-images`; point `sandbox.image` at the GHCR copy in production) with a read-only root, dropped capabilities, a non-root user, and a `noexec` workspace, so `python3 script.py` runs but `./script` does not. With `sandbox.egress` off a container has no network; with it on, it reaches public HTTP and HTTPS only through `bullpen-sandbox-proxy` (`deploy/sandbox/squid.conf`). `just sandbox-images` builds both images locally; `just sandbox-down` removes the containers the engine started.

The sandbox image carries PaddleOCR, Baidu's open-source OCR, in `/opt/paddleocr` with its English and Chinese models downloaded at build time, because the sandbox has no network to fetch them later. `paddle-ocr --lang ch scan.png` prints the text it reads. PaddlePaddle publishes CPU wheels for amd64 only, so an arm64 build skips it (`--build-arg PADDLEOCR=false` skips it anywhere) and scans fall back to tesseract, which has English and Chinese data. `agent.upload_ocr_lang` picks the language attached scans are read in.

The runtime is probed at boot. `sandbox.required = true` refuses to start without one; `false` leaves `run_sandbox` unregistered.

## Images

On push to `main`, CD builds and pushes `ghcr.io/<owner>/bullpen-rust`, where the owner is the account the workflow runs under, `bullpen-sandbox`, `bullpen-sandbox-proxy` and `bullpen-runpod`, tagged `<sha>` and `main`, only for images whose inputs changed (`workflow_dispatch` rebuilds all).

## Observability

- Traces, LLM generations, product events: self-hosted Phoenix, below. Every app exports OpenTelemetry to `BULLPEN_TELEMETRY__PHOENIX_URL` (compose sets it from `BULLPEN_PHOENIX_URL`); an empty URL disables export. Spans land in the project named by `telemetry.project_name`. A Discord conversation is one `session.id`, and each product event is its own span.
- Logs: pretty in development, JSON to stdout otherwise. The developer console also writes `.bullpen/logs/`; deployed logs stay with the platform log driver.
- Metrics: `just metrics` starts Prometheus (:9090) and Grafana (:3000, dashboard **Bullpen inference**), both on loopback. They scrape `llama-server` (`chat` runs with `--metrics`). On a GPU host `just gpu-metrics` adds utilisation, VRAM and temperature panels; the exporter runs under the nvidia container runtime with the `utility` driver capability.

Phoenix holds one `llm` span per model call (full prompt, full reply, token counts, latency) inside a trace tree per conversation. Prometheus holds server-side time series: throughput, queue depth, batching.

### Phoenix

```bash
just phoenix       # trace UI on http://localhost:6006, loopback
```

One container, `arizephoenix/phoenix:version-20.11.0`, data in the `phoenixdata` volume, UI and OTLP endpoint on one port. Export is off until `BULLPEN_TELEMETRY__PHOENIX_URL=http://localhost:6006` is in `.env`. For the compose apps set `BULLPEN_PHOENIX_URL=http://phoenix:6006`. Phoenix has no authentication by default and holds full prompts and replies, so in production it has no host port. When it does authenticate, set `BULLPEN_TELEMETRY__PHOENIX_API_KEY`; every app sends it as a bearer token. `GET /v1/projects/<project>/spans` reads the `llm` spans back, for example to build a fine-tuning set.

### Reading the dashboard

| Panel | Metric | Reads as |
|---|---|---|
| Generation throughput | `rate(llamacpp:tokens_predicted_total[1m])` | tokens per wall-clock second, all requests combined |
| Prompt throughput | `llamacpp:prompt_tokens_total`, `..._cached_total` | prompt tokens evaluated vs. reused from cache |
| Queue | `llamacpp:requests_processing`, `..._deferred` | deferred above zero means requests are waiting for a slot |
| Batching efficiency | `llamacpp:n_busy_slots_per_decode` | near 1 with a non-empty queue means `--parallel` is too low |
| Prompt cache hit rate | cached / (cached + new) | a stable system prompt keeps this high |
| Server-reported speed | `llamacpp:predicted_tokens_seconds`, `..._prompt_tokens_seconds` | llama-server's own running average, independent of load |
| Context ceiling | `llamacpp:n_tokens_max` | the largest context one slot accepts |

The chat server starts with two slots (`BULLPEN_CHAT_PARALLEL`). Each slot takes a `--ctx-size / N` share of the context window, so raise the context with the slots.

Grafana's admin password comes from `BULLPEN_GRAFANA_PASSWORD` (compose default `admin`). Both ports bind to `127.0.0.1`; set a real password before exposing either.
