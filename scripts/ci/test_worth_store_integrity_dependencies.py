#!/usr/bin/env python3

import importlib.util
import pathlib
import unittest


SCRIPT = pathlib.Path(__file__).with_name("check_worth_store_integrity_dependencies.py")
SPEC = importlib.util.spec_from_file_location("c9_integrity_guard", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
GUARD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GUARD)


class IntegrityDependencyGuardTests(unittest.TestCase):
    def test_wire_dependencies_do_not_admit_runtime_or_owner_parsers(self) -> None:
        self.assertEqual(GUARD.OBSERVER_DEPENDENCIES, {
            "worth-foundational", "worth-store-physical-format", "serde", "serde_json",
        })
        for owner in ("worth-store", "worth-store-physical-integrity", "worth-store-wal",
                      "worth-store-recovery-runtime", "worth-store-offline-verifier"):
            self.assertNotIn(owner, GUARD.OBSERVER_DEPENDENCIES)

    def test_declaration_import_is_the_only_allowed_physical_format_route(self) -> None:
        allowed = "use worth_store_physical_format::integrity_declarations::families;"
        self.assertEqual(GUARD.forbidden_format_routes(allowed), [])

        forbidden = "use worth_store_physical_format::wal_frame::decode;"
        self.assertEqual(GUARD.forbidden_format_routes(forbidden), [1])

        aliased = "use worth_store_physical_format as runtime_format;"
        self.assertEqual(GUARD.forbidden_format_routes(aliased), [1])


if __name__ == "__main__":
    unittest.main()
