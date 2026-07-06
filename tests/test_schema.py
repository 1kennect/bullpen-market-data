import unittest
from test_weather import load_weather_tool


class WeatherSchemaTests(unittest.TestCase):
    def test_tool_name(self):
        self.assertEqual(load_weather_tool().name, 'get_weather')

    def test_city_is_required(self):
        schema = load_weather_tool().args_schema.model_json_schema()
        self.assertIn('city', schema['required'])

    def test_city_is_string(self):
        schema = load_weather_tool().args_schema.model_json_schema()
        self.assertEqual(schema['properties']['city']['type'], 'string')


if __name__ == '__main__':
    unittest.main()
