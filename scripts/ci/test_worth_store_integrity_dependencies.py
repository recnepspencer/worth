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

    def test_only_tests_enable_a_limit_owners_mint(self) -> None:
        owners = sorted(GUARD.LIMIT_OWNERS)
        test_only = {
            "dev-dependencies": {
                owner: {"workspace": True, "features": ["test-support"]} for owner in owners
            },
            "features": {"test-support": ["worth-store-physical-backend/test-support"]},
        }
        self.assertEqual(GUARD.production_test_support(test_only), [])
        for table in ("dependencies", "build-dependencies"):
            for owner in owners:
                edge = {table: {owner: {"workspace": True, "features": ["test-support"]}}}
                self.assertEqual(GUARD.production_test_support(edge), [owner])
        renamed = {"dependencies": {"physics": {
            "package": "worth-store-recovery-physics", "features": ["test-support"],
        }}}
        self.assertEqual(GUARD.production_test_support(renamed), ["worth-store-recovery-physics"])
        targeted = {"target": {"cfg(unix)": {"dependencies": {
            "worth-store": {"features": ["test-support"]},
        }}}}
        self.assertEqual(GUARD.production_test_support(targeted), ["worth-store"])
        inherited = {"workspace": {"dependencies": {
            "worth-store-physical-integrity": {"path": "x", "features": ["test-support"]},
        }}}
        self.assertEqual(
            GUARD.production_test_support(inherited), ["worth-store-physical-integrity"]
        )
        by_default = {"package": {"name": "worth-store"}, "features": {"default": [
            "test-support", "worth-store-physical-backend?/test-support", "other/test-support",
        ]}}
        self.assertEqual(
            GUARD.production_test_support(by_default),
            ["worth-store", "worth-store-physical-backend"],
        )
        own_default = {"package": {"name": "other"}, "features": {"default": ["test-support"]}}
        self.assertEqual(GUARD.production_test_support(own_default), [])
        unrelated = {"dependencies": {"other": {"features": ["test-support"]}, "plain": "1"}}
        self.assertEqual(GUARD.production_test_support(unrelated), [])


if __name__ == "__main__":
    unittest.main()
