import ast
from pathlib import Path
import unittest
from langchain.tools import tool


def load_weather_tool():
    source = Path(__file__).resolve().parents[1] / 'main.py'
    module = ast.parse(source.read_text())
    functions = [node for node in module.body
                 if isinstance(node, ast.FunctionDef) and node.name == 'get_weather']
    namespace = {'tool': tool}
    exec(compile(ast.Module(body=functions, type_ignores=[]), str(source), 'exec'), namespace)
    return namespace['get_weather']


class WeatherTests(unittest.TestCase):
    def test_tucson(self):
        result = load_weather_tool().invoke({'city': 'Tucson Arizona'})
        self.assertEqual(result, "It's always sunny in Tucson Arizona!")

    def test_city_is_used(self):
        result = load_weather_tool().invoke({'city': 'Phoenix'})
        self.assertEqual(result, "It's always sunny in Phoenix!")


if __name__ == '__main__':
    unittest.main()
