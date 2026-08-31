# Bullpen weather agent

This Python example connects a Groq chat model to a LangChain weather tool.
The tool returns a deterministic sunny forecast for the requested city.

## Development environment

Use Python 3.12 and run commands from the repository root.

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements.txt
.venv/bin/python -m pip check
```

Keep the virtual environment and local dotenv files out of Git.
The direct runtime dependencies are pinned in requirements.txt.

## Provider settings

Supply GROQ_API_KEY through secure runtime configuration.
Do not print or commit its value.
The entrypoint loads dotenv settings before constructing the model.
Live execution requires HTTPS access to api.groq.com and access to
the configured openai/gpt-oss-120b model.
Use the existing proxy and TLS certificate configuration.

## Running the example

```sh
.venv/bin/python main.py
```

The prototype CLI also accepts --city followed by a quoted place name.
The compact standalone variant uses the fixed Tucson Arizona question.
Successful execution prints the conversation, including weather-tool output
and a final assistant response.

## Agent and tool cycle

The user question is supplied as a message with the user role.
The model receives the get_weather tool schema and can request a city lookup.
LangChain executes the decorated Python tool and appends its result as a
ToolMessage. The model then receives that result and produces a final answer.
The console transcript includes message types, tool-call metadata, and content.

## Offline checks

The prototype tests extract the weather function without creating a client.
They check forecast strings, required city arguments, and message payloads.
The standalone smoke check substitutes a deterministic chat model while
executing the real tool and unchanged application through runpy.
Mocked checks do not establish live credentials or provider model access.
