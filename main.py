from dotenv import load_dotenv
from langchain.agents import create_agent
from langchain_groq import ChatGroq
from langchain.tools import tool
from langchain.tools import tool
import os



#from langchain import ChatOpenAI
#api_key = os.getenv("OPENAI_API_KEY")

load_dotenv()

@tool
def get_weather(city: str) -> str:
    """Get the current weather for a city."""
    return f"It's always sunny in {city}!"

model = ChatGroq(
    model="openai/gpt-oss-120b",
    temperature=0
)

agent = create_agent(
    model=model,
    tools=[get_weather],
    system_prompt="You are a helpful assistant."
)

result = agent.invoke(
    {
        "messages": [
            {
                "role": "user",
                "content": "What's the weather in Tucson Arizona?"
            }
        ]
    }
)

for message in result["messages"]:
    print(type(message).__name__, getattr(message, "tool_calls", None), message.content)

#print(result["messages"][-1].content)