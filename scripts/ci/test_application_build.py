"""The ordinary build entry point issues fresh input without changing the caller."""
import os
from pathlib import Path
import sys
import unittest
from unittest.mock import patch
import uuid

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from application_build import application_build_environment, main


class ApplicationBuildInput(unittest.TestCase):
    def test_each_build_replaces_inherited_identity_and_preserves_other_inputs(self):
        parent = {"WORTH_IMPLEMENTATION_BUILD_ID": "a-stale-build", "UNKNOWN": "kept",
                  "EMPTY": "", "WIDE": "\u03bb\U0001f642"}
        first = application_build_environment(parent)
        second = application_build_environment(parent)
        for issued in (first, second):
            identity = uuid.UUID(issued["WORTH_IMPLEMENTATION_BUILD_ID"])
            self.assertNotEqual(identity.int, 0)
            self.assertEqual(identity.version, 4)
            self.assertEqual({key: issued[key] for key in ("UNKNOWN", "EMPTY", "WIDE")},
                             {key: parent[key] for key in ("UNKNOWN", "EMPTY", "WIDE")})
        self.assertNotEqual(first["WORTH_IMPLEMENTATION_BUILD_ID"],
                            second["WORTH_IMPLEMENTATION_BUILD_ID"])
        self.assertEqual(parent["WORTH_IMPLEMENTATION_BUILD_ID"], "a-stale-build")

    def test_build_command_receives_identity_and_keeps_arguments_and_exit(self):
        command = ["cargo", "build", "--locked", "--manifest-path", "an app/Cargo.toml"]
        with patch.dict(os.environ, {"WORTH_IMPLEMENTATION_BUILD_ID": "old", "BUILD_INPUT": "kept"}), \
             patch("application_build.subprocess.run") as child:
            child.return_value.returncode = 7
            self.assertEqual(main(["--", *command]), 7)
            args, kwargs = child.call_args
            self.assertEqual(args, (command,))
            self.assertEqual(kwargs["env"]["BUILD_INPUT"], "kept")
            self.assertNotEqual(kwargs["env"]["WORTH_IMPLEMENTATION_BUILD_ID"], "old")
            self.assertEqual(os.environ["WORTH_IMPLEMENTATION_BUILD_ID"], "old")


if __name__ == "__main__":
    unittest.main()
