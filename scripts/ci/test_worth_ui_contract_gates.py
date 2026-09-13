from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

import check_worth_ui_appearance_native_matrix as matrix_gate
import check_worth_ui_docs_links as docs_gate
import check_worth_ui_protocol_manifest as protocol_gate
import check_worth_ui_removal_inventory as removal_gate


class WorthUiContractGateTests(unittest.TestCase):
    def test_removal_inventory_detects_exact_count_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "workspaces/worth-ui/sample.rs"
            source.parent.mkdir(parents=True)
            source.write_text(self.removal_source(extra_static_paint=True), encoding="utf-8")
            manifest = self.write_removal_manifest(root)
            with self.assertRaisesRegex(ValueError, "static-paint authority: observed 1"):
                removal_gate.validate(root, manifest)

    def test_removal_inventory_counts_tracked_and_untracked_rust_and_excludes_deleted_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tracked = root / "workspaces/worth-ui/tracked.rs"
            untracked = root / "workspaces/worth-ui/untracked.rs"
            tracked.parent.mkdir(parents=True)
            tracked.write_text("ThemeColorValue", encoding="utf-8")
            untracked.write_text("ThemeColorValue", encoding="utf-8")
            manifest = self.write_removal_manifest(root, tracked_paths=[tracked])

            tracked.unlink()
            with self.assertRaisesRegex(ValueError, "string-backed ThemeColorValue: observed 1"):
                removal_gate.validate(root, manifest)
            untracked.unlink()
            removal_gate.validate(root, manifest)

    def test_removal_inventory_does_not_allow_source_and_manifest_to_self_authorize_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "workspaces/worth-ui/sample.rs"
            source.parent.mkdir(parents=True)
            source.write_text(self.removal_source(extra_static_paint=True), encoding="utf-8")
            manifest = self.write_removal_manifest(root)
            contract = json.loads(manifest.read_text(encoding="utf-8"))
            static_paint = contract["entries"][0]
            static_paint["current_remaining"] += 1
            manifest.write_text(json.dumps(contract), encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "unexpected removal inventory contract"):
                removal_gate.validate(root, manifest)

    def test_removal_inventory_keeps_original_and_gate_zero_counts_distinct(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "workspaces/worth-ui/sample.rs"
            source.parent.mkdir(parents=True)
            source.write_text(self.removal_source(), encoding="utf-8")
            manifest = self.write_removal_manifest(root)
            contract = json.loads(manifest.read_text(encoding="utf-8"))
            color = next(
                entry for entry in contract["entries"]
                if entry["family"] == "string-backed ThemeColorValue"
            )
            color["original_baseline"] = color["gate_zero_remaining"]
            manifest.write_text(json.dumps(contract), encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "unexpected removal inventory contract"):
                removal_gate.validate(root, manifest)

    def test_removal_inventory_rejects_malformed_contracts(self) -> None:
        cases = [
            ("missing family", lambda contract: contract["entries"].pop(), "every required"),
            (
                "duplicate family",
                lambda contract: contract["entries"].append(dict(contract["entries"][0])),
                "duplicate inventory family",
            ),
            (
                "wrong family",
                lambda contract: contract["entries"][0].update(family="unknown"),
                "unexpected removal inventory contract",
            ),
            (
                "wrong pattern",
                lambda contract: contract["entries"][0].update(glob="workspaces/worth-ui/*.rs"),
                "unexpected removal inventory contract",
            ),
            (
                "invalid current count",
                lambda contract: contract["entries"][0].update(current_remaining=-1),
                "must be a non-negative integer",
            ),
            (
                "missing entry key",
                lambda contract: contract["entries"][0].pop("current_retention"),
                "entry keys must remain exact",
            ),
            (
                "wrong stage",
                lambda contract: contract.update(current_stage="gate_3"),
                "current stage must be gate_5_cutover",
            ),
            (
                "target drift",
                lambda contract: contract.update(cutover_target=1),
                "cutover target must be exactly zero",
            ),
        ]
        for name, mutate, message in cases:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                source = root / "workspaces/worth-ui/sample.rs"
                source.parent.mkdir(parents=True)
                source.write_text(self.removal_source(), encoding="utf-8")
                manifest = self.write_removal_manifest(root)
                contract = json.loads(manifest.read_text(encoding="utf-8"))
                mutate(contract)
                manifest.write_text(json.dumps(contract), encoding="utf-8")

                with self.assertRaisesRegex(ValueError, message):
                    removal_gate.validate(root, manifest)

    @staticmethod
    def removal_source(
        *, extra_static_paint: bool = False,
    ) -> str:
        lines = []
        for literal, _, _, _, current_remaining, _ in removal_gate.REQUIRED_FAMILIES.values():
            count = current_remaining + int(
                extra_static_paint and literal == "ComponentStaticPaintContract"
            )
            lines.extend(literal for _ in range(count))
        return "\n".join(lines)

    @staticmethod
    def write_removal_manifest(root: Path, *, tracked_paths: list[Path] | None = None) -> Path:
        entries = [
            {
                "family": family,
                "glob": removal_gate.RUST_GLOB,
                "literal": literal,
                "original_baseline": original_baseline,
                "gate_zero_remaining": gate_zero_remaining,
                "gate_zero_retention": gate_zero_retention,
                "current_remaining": current_remaining,
                "current_retention": current_retention,
            }
            for family, (
                literal,
                original_baseline,
                gate_zero_remaining,
                gate_zero_retention,
                current_remaining,
                current_retention,
            ) in removal_gate.REQUIRED_FAMILIES.items()
        ]
        manifest = root / "inventory.json"
        manifest.write_text(
            json.dumps({
                "current_stage": removal_gate.CURRENT_STAGE,
                "cutover_target": 0,
                "entries": entries,
            }),
            encoding="utf-8",
        )
        subprocess.run(["git", "init", "--quiet"], cwd=root, check=True)
        for tracked_path in tracked_paths or []:
            subprocess.run(
                ["git", "add", tracked_path.relative_to(root).as_posix()],
                cwd=root,
                check=True,
            )
        return manifest

    def test_protocol_manifest_detects_live_source_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            protocol = root / "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_frame/protocol.rs"
            protocol.parent.mkdir(parents=True)
            protocol.write_text("""
                COMPATIBLE_FLOOR: u16 = 7; CURRENT: u16 = 9;
                CURRENT_FRAME_SCHEMA: u16 = 6; CURRENT_PRESENTATION_SCHEMA: u16 = 6;
                CURRENT_OBSERVATION_SCHEMA: u16 = 7; CURRENT_MEASUREMENT_SCHEMA: u16 = 5;
                CURRENT_SOLICITED_EFFECT_SCHEMA: u16 = 1;
            """, encoding="utf-8")
            text = root / "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_projection/semantic_text.rs"
            text.parent.mkdir(parents=True)
            text.write_text("pub const fn current() -> Self { Self(4) }", encoding="utf-8")
            profile = root / "workspaces/worth-ui/crates/worth-ui-host-native/profiles/worth-ui-windows-dx12-v2.toml"
            profile.parent.mkdir(parents=True)
            profile.write_text(
                "identity='worth-ui-windows-dx12-v2'\n"
                "profile_stage='current'\n"
                "live_emission='enabled'\n",
                encoding="utf-8",
            )
            manifest = root / "protocol.json"
            manifest.write_text(json.dumps({"live": protocol_gate.EXPECTED_LIVE}), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "protocol_current drifted"):
                protocol_gate.validate(root, manifest)

    def test_protocol_manifest_rejects_joint_source_and_manifest_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            protocol = root / "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_frame/protocol.rs"
            protocol.parent.mkdir(parents=True)
            protocol.write_text("""
                COMPATIBLE_FLOOR: u16 = 7; CURRENT: u16 = 8;
                CURRENT_FRAME_SCHEMA: u16 = 6; CURRENT_PRESENTATION_SCHEMA: u16 = 6;
                CURRENT_OBSERVATION_SCHEMA: u16 = 7; CURRENT_MEASUREMENT_SCHEMA: u16 = 5;
                CURRENT_SOLICITED_EFFECT_SCHEMA: u16 = 1;
            """, encoding="utf-8")
            text = root / "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_projection/semantic_text.rs"
            text.parent.mkdir(parents=True)
            text.write_text("pub const fn current() -> Self { Self(4) }", encoding="utf-8")
            profile = root / "workspaces/worth-ui/crates/worth-ui-host-native/profiles/worth-ui-windows-dx12-v2.toml"
            profile.parent.mkdir(parents=True)
            profile.write_text(
                "identity='worth-ui-windows-dx12-v2'\n"
                "profile_stage='current'\n"
                "live_emission='enabled'\n",
                encoding="utf-8",
            )
            manifest = root / "protocol.json"
            advanced = dict(protocol_gate.EXPECTED_LIVE)
            advanced["protocol_current"] = 8
            manifest.write_text(json.dumps({"live": advanced}), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "live manifest must be exact"):
                protocol_gate.validate(root, manifest)

    def test_document_gate_rejects_manifest_omission(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = root / "docs.json"
            manifest.write_text(json.dumps({
                "continuing_documents": docs_gate.EXPECTED_CONTINUING_DOCUMENTS[:-1],
                "planned_documents": docs_gate.EXPECTED_PLANNED_DOCUMENTS,
            }), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "document set must remain exact"):
                docs_gate.validate(root, manifest)

    def test_document_gate_rejects_broken_relative_link(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            document = root / "_docs/worth-ui/readme.md"
            document.parent.mkdir(parents=True)
            document.write_text("[missing](missing.md)", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "broken local link"):
                docs_gate.validate(root)

    def test_native_matrix_rejects_prefix_and_comment_symbol_forgeries(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = self.write_native_matrix_fixture(root)
            contract = root / "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_projection/appearance/mechanics.rs"
            valid = [
                symbol for symbol in matrix_gate.EXPECTED_SYMBOLS
                if symbol not in {"UiMountedPointerAffordanceMechanic", "UiMountedBackdropMechanic"}
            ]
            contract.write_text(
                "\n".join(f"pub struct {symbol};" for symbol in valid)
                + "\n// pub struct UiMountedPointerAffordanceMechanic;"
                + "\n/* pub enum UiMountedBackdropMechanic {} */"
                + "\npub struct UiMountedPointerAffordanceMechanicSuffix;",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "host contract symbols missing"):
                matrix_gate.validate(root, manifest)

    def test_native_matrix_rejects_empty_live_owner(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = self.write_native_matrix_fixture(root)
            owner = root / matrix_gate.EXPECTED_OWNERS["runtime_appearance_lowering"]
            owner.write_text("// pub(super) fn lower() {}", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "live runtime_appearance_lowering owner declaration is missing"):
                matrix_gate.validate(root, manifest)

    @staticmethod
    def write_native_matrix_fixture(root: Path) -> Path:
        manifest = root / "native-appearance.json"
        manifest.write_text(json.dumps({
            "live_profile": "worth-ui-windows-dx12-v2",
            "mechanics": matrix_gate.EXPECTED_MECHANICS,
            "required_host_contract_symbols": matrix_gate.EXPECTED_SYMBOLS,
            "live_owners": matrix_gate.EXPECTED_OWNERS,
        }), encoding="utf-8")
        profile = root / "workspaces/worth-ui/crates/worth-ui-host-native/profiles/worth-ui-windows-dx12-v2.toml"
        profile.parent.mkdir(parents=True)
        profile.write_text(
            "identity='worth-ui-windows-dx12-v2'\nprofile_stage='current'\nlive_emission='enabled'\n",
            encoding="utf-8",
        )
        contract = root / "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_projection/appearance/mechanics.rs"
        contract.parent.mkdir(parents=True)
        contract.write_text(
            "\n".join(f"pub struct {symbol};" for symbol in matrix_gate.EXPECTED_SYMBOLS),
            encoding="utf-8",
        )
        declarations = {
            "consumed_fact_index": "pub struct UiGraphConsumedFactIndex;",
            "mounted_preview": "pub enum UiMountedPreviewProjection {}",
            "appearance_presentation_work": "pub struct UiMountedAppearancePresentationWork;",
            "runtime_appearance_lowering": "pub(super) fn lower() {}",
            "native_appearance_translation": "pub(crate) enum UiNativeAppearanceCommand {}",
            "headless_appearance_translation": "pub(crate) fn translate() {}",
        }
        for owner, relative in matrix_gate.EXPECTED_OWNERS.items():
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(declarations[owner], encoding="utf-8")
        return manifest


if __name__ == "__main__":
    unittest.main()
