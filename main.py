from dotenv import load_dotenv
from langchain.tools import tool
from langchain_groq import ChatGroq


load_dotenv()


@tool
def get_weather(city: str) -> str:
    """Get the current weather for a city."""
    return f"It's always sunny in {city}!"


def build_model():
    return ChatGroq(model="openai/gpt-oss-120b", temperature=0)


def main():
    print(get_weather.invoke({"city": "Tucson Arizona"}))


if __name__ == "__main__":
    main()
