# deploy/inference

Model serving: `llama-server` from llama.cpp, configured here. Every model speaks the
OpenAI-compatible API, the only surface `apps/engine` uses. Baidu's ERNIE models on Qianfan,
or any other OpenAI-compatible host, work the same way through `BULLPEN_MODEL__BASE_URL`.

`llama-server` serves one model per process, in its own container.

| Role | Default GGUF | Port | Flags |
|---|---|---|---|
| chat | `Qwen/Qwen3-4B-GGUF:Q4_K_M` | 8000 | `--jinja` for tool calls, `--metrics`, `--ctx-size 8192`, `--parallel 2` |

Override it with `BULLPEN_CHAT_GGUF`.

## Images

`bot.max_images` attachments of a message reach the model as image blocks, filtered to
`image/png`, `image/jpeg`, `image/gif` and `image/webp`. The link goes, not the bytes, so the
server running the model has to be able to fetch the Discord CDN.

The chat GGUF above reads text only, so the blocks are sent and ignored. Reading them needs a
vision model and its projector:

```bash
BULLPEN_CHAT_GGUF=ggml-org/Qwen2.5-VL-7B-Instruct-GGUF just model
```

llama-server loads the matching `mmproj` file from the same repository. A vision model is larger
than the text one, so size the card for it or point `[model]` at a hosted vision model instead.
Set `bot.max_images = 0` to stop sending images at all.

## Concurrency

`--parallel N` gives the server N slots. Continuous batching interleaves their decode steps in
one forward pass, so N requests generate at once over one copy of the weights.

`--ctx-size` is the total across slots. Per-slot context is `ctx-size / parallel`, and the KV
cache scales with the total:

```
BULLPEN_CHAT_PARALLEL=4 BULLPEN_CHAT_CTX=16384   # 4 slots x 4096 tokens
```

Aggregate throughput rises sublinearly with N; per-request generation gets slower as slots fill.

`BULLPEN_AGENT__MODEL_SLOTS` in the engine holds the same number. The engine admits that many
model calls at once and queues the rest for `BULLPEN_AGENT__MODEL_QUEUE_WAIT_SECS`, then answers
503. Set it to 0 to remove the engine-side limit and let `llama-server` queue.

## Local

```bash
just model     # starts chat; the GGUF downloads on first run into the modelcache volume
```

Endpoint: `http://localhost:8000/v1`. From inside compose the host is `chat`.

On a 6 GB card chat fits with room to spare: ~3.1 GB (2.5 GB of
weights plus a KV cache of ~576 MB per 4096 tokens). At the default `--ctx-size 8192
--parallel 2` the chat KV cache is ~1.2 GB. Raise `BULLPEN_CHAT_CTX` only with headroom, and
lower `--n-gpu-layers` to spill to CPU.

## Deployed

Same two services on a GPU host with a larger chat GGUF. The Runpod image defaults to
`unsloth/Qwen3.5-9B-GGUF:Q4_K_M` (5.7 GB, reads images) at `--parallel 2 --ctx-size 16384`,
sized for a 12 GB card such as an RTX 3080 Ti. Size the GPU for the weights plus a
KV cache proportional to `--ctx-size`. Rough starting points:

| VRAM | `BULLPEN_CHAT_PARALLEL` | `BULLPEN_CHAT_CTX` |
|---|---|---|
| 6-8 GB | 2 | 8192 |
| 24 GB | 4 | 16384 |
| 40-80 GB | 8 | 32768 |

Raise `BULLPEN_AGENT__MODEL_SLOTS` to match. Past one card's slots, run more `llama-server`
replicas behind a load balancer; each replica holds its own copy of the weights.
