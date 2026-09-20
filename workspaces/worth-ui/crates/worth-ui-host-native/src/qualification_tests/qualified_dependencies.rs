const QUALIFIED_DEPENDENCIES: &[(&str, &str)] = &[
    ("winit", "0.30.13"),
    ("wgpu", "29.0.4"),
    ("pollster", "0.4.0"),
    ("rustybuzz", "0.20.1"),
    ("swash", "0.2.10"),
];

pub(super) fn assert_qualified_dependencies() {
    let crate_manifest = manifest(include_str!("../../Cargo.toml"));
    let workspace_manifest = manifest(include_str!("../../../../Cargo.toml"));
    let qualified = crate_manifest
        .get("package")
        .and_then(|package| package.get("metadata"))
        .and_then(|metadata| metadata.get("worth-ui-qualified-dependencies"))
        .and_then(toml::Value::as_table)
        .expect("qualified dependency metadata");
    let declarations = crate_manifest
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .expect("native dependency declarations");
    let workspace = workspace_manifest["workspace"]["dependencies"]
        .as_table()
        .expect("workspace dependency declarations");
    let windows = crate_manifest["target"]["cfg(windows)"]["dependencies"]
        .as_table()
        .expect("Windows dependency declarations");
    let linux = crate_manifest["target"]["cfg(target_os = \"linux\")"]["dependencies"]
        .as_table()
        .expect("Linux dependency declarations");
    for &(name, version) in QUALIFIED_DEPENDENCIES {
        assert_exact_pin(name, version, qualified, declarations, workspace);
    }
    assert_exact_pin("winsafe", "0.0.28", qualified, windows, workspace);
    assert_exact_pin("windows", "0.61.3", qualified, windows, workspace);
    for entries in [declarations, workspace] {
        assert_dependency_features(entries, "winit", &["rwh_06"]);
        assert_dependency_features(entries, "wgpu", &["std", "parking_lot", "dx12", "wgsl"]);
    }
    assert_dependency_features(workspace, "winsafe", &[]);
    assert_dependency_features(windows, "winsafe", &["advapi", "user"]);
    assert_dependency_features(workspace, "windows", &[]);
    assert_dependency_features(windows, "windows", &["UI_ViewManagement"]);
    assert_dependency_features(linux, "winit", &["rwh_06", "x11"]);
    assert_eq!(qualified["winsafe-features"].as_str(), Some("advapi,user"));
    assert_eq!(qualified["winit-features"].as_str(), Some("rwh_06"));
    assert_eq!(
        qualified["winit-linux-features"].as_str(),
        Some("rwh_06,x11")
    );
    assert_eq!(
        qualified["wgpu-features"].as_str(),
        Some("std,parking_lot,dx12,wgsl")
    );
    assert_eq!(qualified["wgpu-device-features"].as_str(), Some("empty"));
    assert_eq!(
        qualified["wgpu-limits"].as_str(),
        Some("wgpu-29.0.4-Limits::downlevel_defaults().using_resolution(adapter.limits())")
    );
}

fn assert_exact_pin(
    name: &str,
    version: &str,
    qualified: &toml::map::Map<String, toml::Value>,
    declarations: &toml::map::Map<String, toml::Value>,
    workspace: &toml::map::Map<String, toml::Value>,
) {
    let exact_version = format!("={version}");
    assert_eq!(
        qualified.get(name).and_then(toml::Value::as_str),
        Some(version)
    );
    for declaration in [
        workspace.get(name).expect("workspace pin"),
        declarations.get(name).expect("native direct pin"),
    ] {
        let observed = declaration
            .as_str()
            .or_else(|| declaration.get("version").and_then(toml::Value::as_str));
        assert_eq!(observed, Some(exact_version.as_str()));
    }
}

fn assert_dependency_features(
    declarations: &toml::map::Map<String, toml::Value>,
    name: &str,
    expected: &[&str],
) {
    let dependency = &declarations[name];
    assert_eq!(dependency["default-features"].as_bool(), Some(false));
    let features = dependency
        .get("features")
        .and_then(toml::Value::as_array)
        .map(|features| {
            features
                .iter()
                .map(|feature| feature.as_str().expect("feature string"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(features, expected, "{name} feature posture drifted");
}

fn manifest(text: &str) -> toml::Value {
    text.parse().expect("qualified manifest parses")
}
