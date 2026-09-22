const QUALIFIED_DEPENDENCIES: &[(&str, &str)] = &[
    ("winit", "0.30.13"),
    ("wgpu", "29.0.4"),
    ("pollster", "0.4.0"),
    ("rustybuzz", "0.20.1"),
    ("swash", "0.2.10"),
];

/// Which Cargo feature metadata each qualified profile's manifest must agree
/// with.
///
/// Keyed by profile identity and consulted for every entry of
/// `QUALIFIED_PROFILES`, so a profile added without an entry here fails rather
/// than skipping the check.
const PROFILE_FEATURE_AGREEMENTS: &[(&str, &str, &str)] = &[
    (
        "worth-ui-windows-dx12-v2",
        "wgpu-windows-features",
        "winit-features",
    ),
    (
        "worth-ui-linux-wayland-vulkan-v1",
        "wgpu-linux-features",
        "winit-linux-features",
    ),
    (
        "worth-ui-linux-x11-vulkan-v1",
        "wgpu-linux-features",
        "winit-linux-features",
    ),
    (
        "worth-ui-linux-x11-vulkan-software-v1",
        "wgpu-linux-features",
        "winit-linux-features",
    ),
];

pub(super) fn assert_qualified_dependencies() {
    let crate_manifest = manifest(include_str!("../../Cargo.toml"));
    let workspace_manifest = manifest(include_str!("../../../../Cargo.toml"));
    let tables = QualifiedDependencyTables {
        qualified: table(&crate_manifest["package"]["metadata"]["worth-ui-qualified-dependencies"]),
        declarations: table(&crate_manifest["dependencies"]),
        workspace: table(&workspace_manifest["workspace"]["dependencies"]),
        windows: table(&crate_manifest["target"]["cfg(windows)"]["dependencies"]),
        linux: table(&crate_manifest["target"]["cfg(target_os = \"linux\")"]["dependencies"]),
    };
    assert_qualified_pins(&tables);
    assert_qualified_declared_features(&tables);
    assert_qualified_feature_metadata(tables.qualified);
    assert_profile_features_match_metadata(tables.qualified);
}

/// The five manifest tables a qualified dependency record is spread across.
///
/// They are carried together because every pin is proved by comparing them, not
/// by reading any one of them alone.
struct QualifiedDependencyTables<'a> {
    qualified: &'a toml::Table,
    declarations: &'a toml::Table,
    workspace: &'a toml::Table,
    windows: &'a toml::Table,
    linux: &'a toml::Table,
}

fn assert_qualified_pins(tables: &QualifiedDependencyTables<'_>) {
    for &(name, version) in QUALIFIED_DEPENDENCIES {
        assert_exact_pin(
            name,
            version,
            tables.qualified,
            tables.declarations,
            tables.workspace,
        );
    }
    assert_exact_pin(
        "winsafe",
        "0.0.28",
        tables.qualified,
        tables.windows,
        tables.workspace,
    );
    assert_exact_pin(
        "windows",
        "0.61.3",
        tables.qualified,
        tables.windows,
        tables.workspace,
    );
}

/// Asserts the features each target's declaration actually requests.
///
/// The graphics backend is declared per target rather than in the shared base,
/// so a Linux build resolves Vulkan alone and a Windows build DX12 alone. A
/// backend left in the base would resolve into every target and contradict the
/// `graphics_backend` each profile manifest declares.
fn assert_qualified_declared_features(tables: &QualifiedDependencyTables<'_>) {
    for entries in [tables.declarations, tables.workspace] {
        assert_dependency_features(entries, "winit", &["rwh_06"]);
        assert_dependency_features(entries, "wgpu", &["std", "parking_lot", "wgsl"]);
    }
    assert_dependency_features(
        tables.windows,
        "wgpu",
        &["std", "parking_lot", "dx12", "wgsl"],
    );
    assert_dependency_features(tables.workspace, "winsafe", &[]);
    assert_dependency_features(tables.windows, "winsafe", &["user"]);
    assert_dependency_features(tables.workspace, "windows", &[]);
    assert_dependency_features(tables.windows, "windows", &["UI_ViewManagement"]);
    assert_dependency_features(
        tables.linux,
        "winit",
        &[
            "rwh_06",
            "wayland",
            "wayland-dlopen",
            "wayland-csd-adwaita",
            "x11",
        ],
    );
    assert_dependency_features(
        tables.linux,
        "wgpu",
        &["std", "parking_lot", "vulkan", "wgsl"],
    );
}

/// Asserts each profile's declared backend features against the Cargo feature
/// closure its target actually compiles.
///
/// A profile manifest declaring `vulkan` while its target's metadata compiles
/// `dx12` is a contradiction no other gate sees: the manifest side is checked
/// against its closed record and the Cargo side against the dependency tables,
/// but nothing compares the two. Left unchecked it stays latent until the
/// backend is read from the profile, at which point it becomes an adapter
/// enumeration failure on a machine the author may not own.
fn assert_profile_features_match_metadata(qualified: &toml::Table) {
    for profile in &crate::native_profile::QUALIFIED_PROFILES {
        let identity = profile.identity.as_str();
        let (_, wgpu_key, winit_key) = PROFILE_FEATURE_AGREEMENTS
            .iter()
            .find(|(named, _, _)| *named == identity)
            .unwrap_or_else(|| panic!("{identity} declares which feature metadata it agrees with"));
        let declared = manifest(profile.manifest);
        for (manifest_key, metadata_key) in [
            ("graphics_backend_features", wgpu_key),
            ("event_backend_features", winit_key),
        ] {
            let profile_features = declared[manifest_key]
                .as_str()
                .expect("profile declares its feature closure")
                .replace(';', ",");
            assert_eq!(
                profile_features,
                qualified[*metadata_key]
                    .as_str()
                    .expect("metadata declares its feature closure"),
                "{identity} {manifest_key} contradicts Cargo metadata {metadata_key}"
            );
        }
    }
}

fn assert_qualified_feature_metadata(qualified: &toml::Table) {
    for (key, value) in [
        ("winit-features", "rwh_06"),
        (
            "winit-linux-features",
            "rwh_06,wayland,wayland-dlopen,wayland-csd-adwaita,x11",
        ),
        ("wgpu-features", "std,parking_lot,wgsl"),
        ("wgpu-windows-features", "std,parking_lot,dx12,wgsl"),
        ("wgpu-linux-features", "std,parking_lot,vulkan,wgsl"),
        ("wgpu-device-features", "empty"),
        (
            "wgpu-limits",
            "wgpu-29.0.4-Limits::downlevel_defaults().using_resolution(adapter.limits())",
        ),
    ] {
        assert_eq!(qualified[key].as_str(), Some(value), "metadata {key}");
    }
}

fn table(value: &toml::Value) -> &toml::Table {
    value.as_table().expect("qualified dependency table")
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
