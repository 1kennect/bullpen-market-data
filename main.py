from dotenv import load_dotenv
from langchain.tools import tool
from langchain_groq import ChatGroq
from langchain.agents import create_agent


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
    result = run_question("What's the weather in Tucson Arizona?")
    print(result["messages"][-1].content)


if __name__ == "__main__":
    main()
