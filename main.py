from dotenv import load_dotenv
from langchain.tools import tool
from langchain_groq import ChatGroq
from langchain.agents import create_agent
import argparse


load_dotenv()


@tool
def get_weather(city: str) -> str:
    """Get the current weather for a city."""
    return f"It's always sunny in {city}!"


def build_model():
    return ChatGroq(model="openai/gpt-oss-120b", temperature=0)


def build_agent():
    return create_agent(
        model=build_model(), tools=[get_weather],
        system_prompt="You are a helpful assistant.",
    )


def run_question(question: str):
    return build_agent().invoke(
        {"messages": [{"role": "user", "content": question}]}
    )


def main():
    parser = argparse.ArgumentParser(description="Ask the weather agent")
    parser.add_argument("--city", default="Tucson Arizona")
    args = parser.parse_args()
    result = run_question(f"What's the weather in {args.city}?")
    for message in result["messages"]:
        print(type(message).__name__,
              getattr(message, "tool_calls", None), message.content)


if __name__ == "__main__":
    main()
