from unittest import TestCase, main

import check_worth_ui_native_features as native_features


class WorthUiNativeFeatureAuditTests(TestCase):
    def test_every_mode_names_an_explicit_supported_target(self) -> None:
        self.assertEqual(
            native_features.MODES,
            (
                ("linux", native_features.LINUX_TARGET, ()),
                (
                    "linux-all-features",
                    native_features.LINUX_TARGET,
                    ("--all-features",),
                ),
                ("windows", native_features.WINDOWS_TARGET, ()),
                (
                    "windows-all-features",
                    native_features.WINDOWS_TARGET,
                    ("--all-features",),
                ),
            ),
        )

    def test_target_contracts_distinguish_linux_x11_from_windows(self) -> None:
        linux = native_features.EXPECTED_FEATURES[native_features.LINUX_TARGET]
        windows = native_features.EXPECTED_FEATURES[native_features.WINDOWS_TARGET]

        self.assertIn("x11", linux[("winit", "0.30.13")])
        self.assertEqual(windows[("winit", "0.30.13")], {"rwh_06"})
        self.assertNotIn(
            "winsafe",
            native_features.expected_dependencies(native_features.LINUX_TARGET)[
                ("worth-ui-host-native", "0.1.0")
            ],
        )
        self.assertIn(
            "winsafe",
            native_features.expected_dependencies(native_features.WINDOWS_TARGET)[
                ("worth-ui-host-native", "0.1.0")
            ],
        )
        self.assertNotIn(
            "windows",
            native_features.expected_dependencies(native_features.LINUX_TARGET)[
                ("worth-ui-host-native", "0.1.0")
            ],
        )
        self.assertIn(
            "windows",
            native_features.expected_dependencies(native_features.WINDOWS_TARGET)[
                ("worth-ui-host-native", "0.1.0")
            ],
        )

    def test_feature_tree_parser_keeps_the_target_resolved_feature_closure(self) -> None:
        parsed = native_features.parse_feature_tree(
            "unrelated v1.0.0|default\n"
            "winit v0.30.13|rwh_06,x11,x11-dl\n"
            "winit v0.30.13|x11rb\n"
            "wgpu v29.0.4|dx12,parking_lot,std,wgsl\n"
        )

        self.assertEqual(
            parsed[("winit", "0.30.13")],
            {"rwh_06", "x11", "x11-dl", "x11rb"},
        )
        self.assertNotIn(("unrelated", "1.0.0"), parsed)

    def test_linux_validation_rejects_a_missing_x11_closure(self) -> None:
        observed = {
            identity: set(features)
            for identity, features in native_features.EXPECTED_FEATURES[
                native_features.LINUX_TARGET
            ].items()
        }
        observed[("winit", "0.30.13")].remove("x11")

        with self.assertRaisesRegex(ValueError, "linux: winit features drifted"):
            native_features.validate_features(
                observed, "linux", native_features.LINUX_TARGET
            )

    def test_windows_validation_rejects_linux_feature_leakage(self) -> None:
        observed = {
            identity: set(features)
            for identity, features in native_features.EXPECTED_FEATURES[
                native_features.WINDOWS_TARGET
            ].items()
        }
        observed[("winit", "0.30.13")].add("x11")

        with self.assertRaisesRegex(ValueError, "windows: winit features drifted"):
            native_features.validate_features(
                observed, "windows", native_features.WINDOWS_TARGET
            )


if __name__ == "__main__":
    main()
