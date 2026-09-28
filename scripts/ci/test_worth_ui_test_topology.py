from pathlib import Path
from tempfile import TemporaryDirectory
from unittest import TestCase, TextTestRunner, defaultTestLoader

from check_worth_ui_test_topology import package_violations
from worth_ui_compile_contract_topology import compile_reconciliation_violations
from worth_ui_test_source_topology import (
    real_filesystem_proof_violations,
    rust_test_sources,
    source_violations,
)


class WorthUiTestTopologySourceTests(TestCase):
    def test_real_filesystem_proof_rejects_injected_source_and_events(self) -> None:
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            provider = root / "filesystem_provider.rs"
            proof = root / "filesystem_contract.rs"
            provider.write_text("pub struct FilesystemProvider { root: PathBuf }\n", encoding="utf-8")
            proof.write_text("fn real_disk_contract() {}\n", encoding="utf-8")
            config = {
                "filesystem_provider_source": "filesystem_provider.rs",
                "real_filesystem_proof_sources": ["filesystem_contract.rs"],
            }
            self.assertEqual(real_filesystem_proof_violations(root, config), [])

            provider.write_text("fn with_file(source_text: String) {}\n", encoding="utf-8")
            proof.write_text("let event = WorthUiWatcherEvent::modified(path);\n", encoding="utf-8")

            violations = real_filesystem_proof_violations(root, config)

            self.assertTrue(violations)
            self.assertEqual(
                {violation.rule for violation in violations},
                {"filesystem-proof-substitution"},
            )

    def test_nested_test_modules_are_enforced_but_embedded_repositories_are_not(self) -> None:
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            package = root / "package"
            workspace = root / "Cargo.toml"
            nested = package / "tests" / "nested" / "module.rs"
            embedded = (
                package
                / "tests"
                / "fixtures"
                / "topology_negative"
                / "fake.rs"
            )
            nested.parent.mkdir(parents=True)
            embedded.parent.mkdir(parents=True)
            workspace.write_text(
                '[workspace]\nmembers = ["package"]\n',
                encoding="utf-8",
            )
            (package / "Cargo.toml").write_text(
                '[package]\nname = "topology-test"\nversion = "0.1.0"\n',
                encoding="utf-8",
            )
            nested.write_text('const RUSTFLAGS: &str = "-Awarnings";\n', encoding="utf-8")
            embedded.write_text('const RUSTFLAGS: &str = "fixture data";\n', encoding="utf-8")
            config = {
                "workspace_manifest": "Cargo.toml",
                "allowed_trybuild_sessions": [],
                "max_trybuild_sessions": 0,
            }

            sources = rust_test_sources(root, config)
            violations = source_violations(root, config)

            self.assertEqual(sources, [nested])
            self.assertEqual(len(violations), 1)
            self.assertEqual(violations[0].rule, "warning-fingerprint")
            self.assertIn("tests/nested/module.rs", violations[0].detail)

    def test_generated_compilation_forms_are_rejected_from_ordinary_tests(self) -> None:
        hostile_sources = {
            "nested_cargo": 'Command::new("cargo").status();',
            "direct_rustc": 'Command::new("rustc").status();',
            "manifest_arg": 'command.arg("--manifest-path");',
            "generated_manifest": 'std::fs::write(root.join("Cargo.toml"), "[package]");',
        }
        for name, source_text in hostile_sources.items():
            with self.subTest(name=name), TemporaryDirectory() as temporary:
                root, test_source, config = source_workspace(Path(temporary))
                test_source.write_text(source_text, encoding="utf-8")

                violations = source_violations(root, config)

                self.assertIn("generated-compilation", {item.rule for item in violations})

    def test_extra_trybuild_session_is_rejected(self) -> None:
        with TemporaryDirectory() as temporary:
            root, test_source, config = source_workspace(Path(temporary))
            test_source.write_text("trybuild::TestCases::new();", encoding="utf-8")

            violations = source_violations(root, config)

            self.assertIn("trybuild-session", {item.rule for item in violations})

    def test_compile_fail_aggregation_preserves_inventory_coverage(self) -> None:
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            suites = root / "crate/tests/suites"
            fixture_root = root / "crate/tests/ui"
            suites.mkdir(parents=True)
            fixture_root.mkdir(parents=True)
            (fixture_root / "aggregate.rs").write_text(
                'mod covered { include!("covered.rs"); }\nfn main() {}\n',
                encoding="utf-8",
            )
            (fixture_root / "aggregate.stderr").write_text("errors\n", encoding="utf-8")
            (fixture_root / "covered.rs").write_text("compile_error!(\"covered\");\n", encoding="utf-8")
            inventory = (
                "kind,path,legacy_harness\n"
                "fail,tests/ui/aggregate.rs,shared_owner\n"
                "fail,tests/ui/covered.rs,shared_owner\n"
            )
            execution = (
                "kind,path,legacy_harness\n"
                "fail,tests/ui/aggregate.rs,shared_owner\n"
            )
            (suites / "inventory.csv").write_text(inventory, encoding="utf-8")
            (suites / "execution.csv").write_text(execution, encoding="utf-8")
            config = {
                "compile_contract_sessions": {
                    "product": {
                        "inventory": "crate/tests/suites/inventory.csv",
                        "execution": "crate/tests/suites/execution.csv",
                        "structural_replacement_patterns": [],
                    }
                }
            }
            self.assertEqual(compile_reconciliation_violations(root, config), [])
            (fixture_root / "covered.stderr").write_text("optional covered error\n", encoding="utf-8")
            self.assertEqual(compile_reconciliation_violations(root, config), [])

    def test_inventoried_compile_pass_must_be_executed(self) -> None:
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            suites = root / "crate/tests/suites"
            fixture_root = root / "crate/tests/ui"
            suites.mkdir(parents=True)
            fixture_root.mkdir(parents=True)
            (fixture_root / "current.rs").write_text("fn main() {}\n", encoding="utf-8")
            (fixture_root / "stale.rs").write_text("fn main() {}\n", encoding="utf-8")
            inventory = (
                "kind,path,legacy_harness\n"
                "pass,tests/ui/current.rs,facade_owner\n"
                "pass,tests/ui/stale.rs,facade_owner\n"
            )
            execution = (
                "kind,path,legacy_harness\n"
                "pass,tests/ui/current.rs,facade_owner\n"
            )
            (suites / "inventory.csv").write_text(inventory, encoding="utf-8")
            (suites / "execution.csv").write_text(execution, encoding="utf-8")
            config = {
                "compile_contract_sessions": {
                    "product": {
                        "inventory": "crate/tests/suites/inventory.csv",
                        "execution": "crate/tests/suites/execution.csv",
                        "structural_replacement_patterns": [],
                    }
                }
            }

            violations = compile_reconciliation_violations(root, config)

            details = "\n".join(violation.detail for violation in violations)
            self.assertIn("inventoried compile-pass is not executed", details)
            self.assertIn("tests/ui/stale.rs", details)

    def test_compile_fixture_owner_rejects_uninventoried_targets_and_extra_sessions(self) -> None:
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            suites = root / "crate/tests/suites"
            fixture_root = root / "crate/tests/ui"
            owner = root / "compile-owner"
            suites.mkdir(parents=True)
            fixture_root.mkdir(parents=True)
            owner.mkdir()
            case = fixture_root / "case.rs"
            extra = fixture_root / "extra.rs"
            case.write_text("fn main() {}\n", encoding="utf-8")
            extra.write_text("fn main() {}\n", encoding="utf-8")
            rows = "kind,path,legacy_harness\npass,tests/ui/case.rs,owner\n"
            (suites / "inventory.csv").write_text(rows, encoding="utf-8")
            (suites / "execution.csv").write_text(rows, encoding="utf-8")
            (owner / "Cargo.toml").write_text(
                '[[bin]]\nname = "case"\npath = "../crate/tests/ui/case.rs"\n'
                '[[bin]]\nname = "extra"\npath = "../crate/tests/ui/extra.rs"\n',
                encoding="utf-8",
            )
            (root / "runner.py").write_text(
                "cargo_check(failing)\ncargo_check(passing)\ncargo_check(extra)\n",
                encoding="utf-8",
            )
            config = {
                "compile_contract_sessions": {
                    "product": {
                        "inventory": "crate/tests/suites/inventory.csv",
                        "execution": "crate/tests/suites/execution.csv",
                        "structural_replacement_patterns": [],
                    }
                },
                "compile_contract_fixture_manifest": "compile-owner/Cargo.toml",
                "compile_contract_runner": "runner.py",
                "max_compile_cargo_sessions": 2,
            }

            violations = compile_reconciliation_violations(root, config)

            self.assertIn("compile-fixture-owner", {item.rule for item in violations})
            self.assertIn(
                "compile-cargo-session-budget", {item.rule for item in violations}
            )

    def test_unlisted_physical_compile_material_is_rejected(self) -> None:
        hostile_material = {
            "rust fixture": ("extra.rs", "fn main() {}\n", "compile-physical-fixture"),
            "diagnostic": ("extra.stderr", "error fixture\n", "compile-physical-fixture"),
            "workspace": ("Cargo.toml", "[workspace]\n", "generated-compilation"),
        }
        for name, (filename, contents, expected_rule) in hostile_material.items():
            with self.subTest(name=name), TemporaryDirectory() as temporary:
                root = Path(temporary)
                suites = root / "crate/tests/suites"
                fixture_root = root / "crate/tests/ui"
                suites.mkdir(parents=True)
                fixture_root.mkdir(parents=True)
                (fixture_root / "case.rs").write_text("fn main() {}\n", encoding="utf-8")
                rows = "kind,path,legacy_harness\npass,tests/ui/case.rs,owner\n"
                (suites / "inventory.csv").write_text(rows, encoding="utf-8")
                (suites / "execution.csv").write_text(rows, encoding="utf-8")
                config = {
                    "compile_contract_sessions": {
                        "certification": {
                            "inventory": "crate/tests/suites/inventory.csv",
                            "execution": "crate/tests/suites/execution.csv",
                            "structural_replacement_patterns": [],
                        }
                    }
                }
                self.assertEqual(compile_reconciliation_violations(root, config), [])
                (fixture_root / filename).write_text(contents, encoding="utf-8")

                violations = compile_reconciliation_violations(root, config)

                self.assertIn(expected_rule, {item.rule for item in violations})

    def test_unexpected_integration_target_is_rejected(self) -> None:
        with TemporaryDirectory() as temporary:
            root = Path(temporary)
            package = root / "package"
            tests = package / "tests"
            tests.mkdir(parents=True)
            (tests / "allowed.rs").write_text("", encoding="utf-8")
            (tests / "unexpected.rs").write_text("", encoding="utf-8")
            (package / "Cargo.toml").write_text(
                '[package]\nname = "target-proof"\nversion = "0.1.0"\nautotests = false\n'
                '[[test]]\nname = "allowed"\npath = "tests/allowed.rs"\n'
                '[[test]]\nname = "unexpected"\npath = "tests/unexpected.rs"\n',
                encoding="utf-8",
            )
            config = {
                "packages": {
                    "target-proof": {
                        "manifest": "package/Cargo.toml",
                        "max_integration_targets": 2,
                        "expected_integration_targets": ["allowed"],
                    }
                }
            }

            violations, _, _ = package_violations(root, config)

            self.assertEqual(violations[0].rule, "target-regression")
            self.assertIn("unexpected integration target unexpected", violations[0].detail)


def source_workspace(root: Path):
    package = root / "package"
    test_source = package / "tests/contract.rs"
    test_source.parent.mkdir(parents=True)
    (root / "Cargo.toml").write_text('[workspace]\nmembers = ["package"]\n', encoding="utf-8")
    (package / "Cargo.toml").write_text(
        '[package]\nname = "source-proof"\nversion = "0.1.0"\n', encoding="utf-8"
    )
    return root, test_source, {
        "workspace_manifest": "Cargo.toml",
        "allowed_trybuild_sessions": [],
        "max_trybuild_sessions": 0,
    }

if __name__ == "__main__":
    from test_worth_ui_compile_contracts import WorthUiCompileContractRunnerTests

    suite = defaultTestLoader.loadTestsFromTestCase(WorthUiTestTopologySourceTests)
    suite.addTests(
        defaultTestLoader.loadTestsFromTestCase(WorthUiCompileContractRunnerTests)
    )
    result = TextTestRunner().run(suite)
    raise SystemExit(0 if result.wasSuccessful() else 1)
