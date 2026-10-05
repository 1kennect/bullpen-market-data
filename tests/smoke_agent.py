"""Exercise the unchanged application's agent/tool loop without Groq requests."""

from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path
import runpy
from unittest.mock import patch

import langchain_groq
from langchain_core.language_models.chat_models import BaseChatModel
from langchain_core.messages import AIMessage, HumanMessage, SystemMessage, ToolMessage
from langchain_core.outputs import ChatGeneration, ChatResult
from pydantic import PrivateAttr


REPOSITORY = Path(__file__).resolve().parents[1]
QUESTION = "What's the weather in Tucson Arizona?"
WEATHER = "It's always sunny in Tucson Arizona!"
ANSWER = "The weather in Tucson Arizona is sunny."


class OfflineWeatherModel(BaseChatModel):
    _calls: int = PrivateAttr(default=0)
    _tool_names: set[str] = PrivateAttr(default_factory=set)

    @property
    def _llm_type(self):
        return "cloud-onboarding-offline-weather"

    def bind_tools(self, tools, *, tool_choice=None, **kwargs):
        self._tool_names = {tool.name for tool in tools}
        assert self._tool_names == {"get_weather"}, self._tool_names
        return self

    def _generate(self, messages, stop=None, run_manager=None, **kwargs):
        self._calls += 1
        assert isinstance(messages[0], SystemMessage)
        assert messages[0].content == "You are a helpful assistant."
        assert isinstance(messages[1], HumanMessage)
        assert messages[1].content == QUESTION
        if self._calls == 1:
            assert len(messages) == 2
            response = AIMessage(
                content="",
                tool_calls=[
                    {
                        "name": "get_weather",
                        "args": {"city": "Tucson Arizona"},
                        "id": "offline-weather-call",
                        "type": "tool_call",
                    }
                ],
            )
        else:
            assert self._calls == 2, "Unexpected extra model invocation"
            assert len(messages) == 4
            assert isinstance(messages[2], AIMessage)
            assert isinstance(messages[3], ToolMessage)
            assert messages[3].tool_call_id == "offline-weather-call"
            assert messages[3].name == "get_weather"
            assert messages[3].content == WEATHER, messages[3].content
            response = AIMessage(content=ANSWER)
        return ChatResult(generations=[ChatGeneration(message=response)])


def main():
    model = OfflineWeatherModel()
    constructor_calls = []

    def make_model(*args, **kwargs):
        constructor_calls.append((args, kwargs))
        assert args == ()
        assert kwargs == {"model": "openai/gpt-oss-120b", "temperature": 0}
        return model

    output = StringIO()
    with patch.object(langchain_groq, "ChatGroq", side_effect=make_model):
        with redirect_stdout(output):
            namespace = runpy.run_path(str(REPOSITORY / "main.py"), run_name="__main__")

    messages = namespace["result"]["messages"]
    assert len(constructor_calls) == 1
    assert model._calls == 2, model._calls
    assert len(messages) == 4
    assert isinstance(messages[0], HumanMessage)
    assert messages[0].content == QUESTION
    assert isinstance(messages[1], AIMessage)
    assert messages[1].tool_calls[0]["name"] == "get_weather"
    assert isinstance(messages[2], ToolMessage)
    assert messages[2].content == WEATHER
    assert isinstance(messages[3], AIMessage)
    assert messages[3].content == ANSWER
    assert messages[3].tool_calls == []
    assert WEATHER in output.getvalue()
    assert ANSWER in output.getvalue()
    print("PASS: unchanged main.py completed its LangChain agent/tool cycle.")
    print("Groq requests were mocked; live authentication and model access were not tested.")


if __name__ == "__main__":
    main()
