import ast
from pathlib import Path
import unittest


class RecordingAgent:
    def invoke(self, payload):
        self.payload = payload
        return {'messages': payload['messages']}


class InvocationTests(unittest.TestCase):
    def test_human_message_shape(self):
        source = Path(__file__).resolve().parents[1] / 'main.py'
        module = ast.parse(source.read_text())
        nodes = [node for node in module.body
                 if isinstance(node, ast.FunctionDef) and node.name == 'run_question']
        if not nodes:
            self.skipTest('Standalone entrypoint is covered by smoke_agent.py')
        agent = RecordingAgent()
        namespace = {'build_agent': lambda: agent}
        exec(compile(ast.Module(body=nodes, type_ignores=[]), str(source), 'exec'), namespace)
        namespace['run_question']('Weather in Tucson?')
        self.assertEqual(agent.payload, {
            'messages': [{'role': 'user', 'content': 'Weather in Tucson?'}]})


if __name__ == '__main__':
    unittest.main()
