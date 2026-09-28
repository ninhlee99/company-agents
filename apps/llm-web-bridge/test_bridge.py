import json
import unittest

from bridge import make_prompt, normalize


class BridgeContractTests(unittest.TestCase):
    def test_prompt_keeps_company_instructions_and_user_data_separate(self):
        job = {"system": "SYSTEM", "user": "USER"}
        value = make_prompt(job)
        self.assertIn("--- BEGIN SYSTEM ---", value)
        self.assertIn("--- BEGIN USER DATA ---", value)
        self.assertLess(value.index("SYSTEM"), value.index("USER"))

    def test_normalize_requires_json_object(self):
        self.assertEqual(normalize('{"ok":true}'), '{"ok":true}')
        with self.assertRaises(ValueError):
            normalize('["not-object"]')

    def test_normalize_rejects_invalid_json(self):
        with self.assertRaises(json.JSONDecodeError):
            normalize("not-json")


if __name__ == "__main__":
    unittest.main()
