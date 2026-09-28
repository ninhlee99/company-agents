import json
import unittest

from bridge import BACKENDS, make_prompt, normalize


class BridgeContractTests(unittest.TestCase):
    def test_only_supported_web_backends_exist(self):
        self.assertEqual(set(BACKENDS), {"chatgpt-web", "gemini-web", "claude-web"})

    def test_prompt_keeps_system_and_user_boundaries(self):
        prompt = make_prompt({
            "system": "SYSTEM",
            "user": "USER"
        })
        self.assertIn("BEGIN SYSTEM", prompt)
        self.assertIn("BEGIN USER DATA", prompt)
        self.assertIn("SYSTEM", prompt)
        self.assertIn("USER", prompt)
        self.assertIn("Do not call tools", prompt)

    def test_normalize_accepts_json_objects(self):
        output = normalize('{"action":"ProduceReport","confidence":0.8}')
        self.assertEqual(json.loads(output)["action"], "ProduceReport")

    def test_normalize_rejects_non_object(self):
        with self.assertRaises(ValueError):
            normalize('["not-an-object"]')

    def test_normalize_rejects_invalid_json(self):
        with self.assertRaises(ValueError):
            normalize("not json")


if __name__ == "__main__":
    unittest.main()
