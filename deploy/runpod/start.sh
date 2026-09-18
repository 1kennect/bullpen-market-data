#!/usr/bin/env bash
# Prepares the model cache, then runs supervisord: the chat model server, the store, the engine and
# the bot. The store reads BULLPEN_STORE__DATABASE_URL, a Postgres outside the pod. With
# BULLPEN_POD_ROLE=models only the model server runs, for apps that run elsewhere.
set -euo pipefail

data="${BULLPEN_DATA_DIR:-/workspace}"

if [ -n "${BULLPEN_MODELS_API_KEY:-}" ]; then
    export BULLPEN_MODELS_HOST=0.0.0.0
    export BULLPEN_MODEL__API_KEY="$BULLPEN_MODELS_API_KEY"
else
    export BULLPEN_MODELS_HOST=127.0.0.1
fi

export BULLPEN_RUN_APPS=true
if [ "${BULLPEN_POD_ROLE:-all}" = "models" ]; then
    if [ -z "${BULLPEN_MODELS_API_KEY:-}" ]; then
        echo "BULLPEN_POD_ROLE=models needs BULLPEN_MODELS_API_KEY" >&2
        exit 1
    fi
    export BULLPEN_RUN_APPS=false
fi

mkdir -p "$data/models"
exec /usr/bin/supervisord -c /etc/bullpen/supervisord.conf
