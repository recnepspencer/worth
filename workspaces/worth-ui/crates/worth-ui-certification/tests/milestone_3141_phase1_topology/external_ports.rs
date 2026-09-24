use syn::visit::Visit;

#[test]
fn phase_two_external_effect_ports_are_concrete_and_vendor_scoped() {
    let inventory = super::workspace_source_inventory();
    for (contract_path, contract, implementation_path, implementation) in [
        (
            "crates/worth-ui-host-native/src/native/event_loop/window_port.rs",
            "trait UiNativeWindowPort",
            "crates/worth-ui-host-native/src/native/event_loop/window_port.rs",
            "impl UiNativeWindowPort for UiWinitNativeWindowPort",
        ),
        (
            "crates/worth-ui-host-native/src/native/graphics/backend/port.rs",
            "trait UiNativeGraphicsPort",
            "crates/worth-ui-host-native/src/native/graphics/backend/wgpu.rs",
            "impl UiNativeGraphicsPort for UiWgpuNativeGraphicsPort",
        ),
        (
            "crates/worth-ui-host-native/src/native/presentation/port.rs",
            "trait UiNativePresentationPort",
            "crates/worth-ui-host-native/src/native/presentation/port.rs",
            "impl UiNativePresentationPort for UiWgpuNativePresentationPort",
        ),
    ] {
        let contract_source = inventory
            .source(contract_path)
            .expect("external effect port contract owner");
        assert!(
            contract_source.text().contains(contract),
            "{contract_path} omits {contract}"
        );
        let implementation_source = inventory
            .source(implementation_path)
            .expect("external effect port implementation owner");
        assert!(
            implementation_source.text().contains(implementation),
            "{implementation_path} omits {implementation}"
        );
    }

    assert_port_outputs_are_mechanical(inventory);
    assert_native_vendors_are_confined(inventory);
}

fn assert_native_vendors_are_confined(
    inventory: &worth_ui_certification::topology::WorkspaceSourceInventory,
) {
    let violations = ["crates", "apps"]
        .into_iter()
        .flat_map(|root| inventory.rust_files_under(root))
        .filter(|source| {
            let path = source.relative_path().to_string_lossy().replace('\\', "/");
            !is_test_source(&path)
                && uses_native_vendor_path(source.text())
                && !is_approved_vendor_owner(&path)
        })
        .map(|source| source.relative_path().to_string_lossy().replace('\\', "/"))
        .collect::<Vec<_>>();
    assert!(
        violations.is_empty(),
        "wgpu/winit escaped the native adapter owners: {violations:?}"
    );
}

// A `*_tests/` directory holds the child modules of a `*_tests.rs` module.
fn is_test_source(path: &str) -> bool {
    path.ends_with("_tests.rs")
        || path.ends_with("/tests.rs")
        || path.contains("/tests/")
        || path.contains("_tests/")
}

fn is_approved_vendor_owner(path: &str) -> bool {
    let module_owners = [
        "crates/worth-ui-host-native/src/native/event_loop",
        "crates/worth-ui-host-native/src/native/graphics",
        "crates/worth-ui-host-native/src/native/input",
        "crates/worth-ui-host-native/src/native/lifecycle_protocol",
        "crates/worth-ui-host-native/src/native/platform",
        "crates/worth-ui-host-native/src/native/presentation",
        "crates/worth-ui-host-native/src/native/readiness",
    ];
    module_owners.iter().any(|owner| {
        path == format!("{owner}.rs") || path.starts_with(&format!("{owner}/"))
    }) || [
        "crates/worth-ui-host-native/src/native/capture/readback.rs",
        "crates/worth-ui-host-native/src/native/lifecycle/orchestrator/input.rs",
        "crates/worth-ui-host-native/src/native/lifecycle/presentation_access.rs",
        "crates/worth-ui-host-native/src/native/mechanics_adapter/text_atlas_dx12_upload_port.rs",
        "crates/worth-ui-host-native/src/native/mechanics_adapter/text_atlas_upload.rs",
        "crates/worth-ui-host-native/src/native/text_atlas/upload.rs",
        "crates/worth-ui-host-native/src/native/text_atlas/upload_batch.rs",
        "crates/worth-ui-host-native/src/native/text_atlas/upload_staging.rs",
    ]
    .contains(&path)
}

fn uses_native_vendor_path(source: &str) -> bool {
    struct VendorPaths(bool);
    impl<'ast> Visit<'ast> for VendorPaths {
        fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
            self.0 |= item.ident == "wgpu" || item.ident == "winit";
            syn::visit::visit_item_extern_crate(self, item);
        }

        fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
            fn mentions_vendor(tree: &syn::UseTree) -> bool {
                match tree {
                    syn::UseTree::Path(path) => path.ident == "wgpu" || path.ident == "winit",
                    syn::UseTree::Name(name) => name.ident == "wgpu" || name.ident == "winit",
                    syn::UseTree::Rename(rename) => {
                        rename.ident == "wgpu" || rename.ident == "winit"
                    }
                    syn::UseTree::Group(group) => group.items.iter().any(mentions_vendor),
                    syn::UseTree::Glob(_) => false,
                }
            }
            self.0 |= mentions_vendor(&item.tree);
            syn::visit::visit_item_use(self, item);
        }

        fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
            let test_only = item.attrs.iter().any(|attribute| {
                attribute.path().is_ident("cfg")
                    && attribute.meta.require_list().is_ok_and(|list| {
                        list.tokens
                            .to_string()
                            .split_whitespace()
                            .any(|token| token == "test")
                    })
            });
            if !test_only {
                syn::visit::visit_item_mod(self, item);
            }
        }

        fn visit_path(&mut self, path: &'ast syn::Path) {
            self.0 |= path
                .segments
                .first()
                .is_some_and(|segment| segment.ident == "wgpu" || segment.ident == "winit");
            syn::visit::visit_path(self, path);
        }
    }
    let syntax = syn::parse_file(source).expect("native source parses");
    let mut paths = VendorPaths(false);
    paths.visit_file(&syntax);
    paths.0
}

fn assert_port_outputs_are_mechanical(
    inventory: &worth_ui_certification::topology::WorkspaceSourceInventory,
) {
    for path in [
        "crates/worth-ui-host-native/src/native/event_loop/window_port.rs",
        "crates/worth-ui-host-native/src/native/graphics/backend/port.rs",
        "crates/worth-ui-host-native/src/native/presentation/port.rs",
    ] {
        let source = inventory.source(path).expect("external port source").text();
        for forbidden in [
            "UiHostSurfacePresentationOutcome",
            "UiNativeEffectPosture",
            "RejectedBeforeEffects",
            "PresentationIndeterminate",
            "InputRejected",
            "CapacityUnavailable",
        ] {
            assert!(!source.contains(forbidden), "{path} mints {forbidden}");
        }
    }
    for (path, denial) in [
        (
            "crates/worth-ui-host-native/src/native/event_loop/window_port.rs",
            "UiNativeWindowPortDenial",
        ),
        (
            "crates/worth-ui-host-native/src/native/graphics/backend/port.rs",
            "UiNativeGraphicsPortDenial",
        ),
        (
            "crates/worth-ui-host-native/src/native/presentation/port.rs",
            "UiNativePresentationPortFailure",
        ),
    ] {
        assert!(
            inventory
                .source(path)
                .expect("external port source")
                .text()
                .contains(denial),
            "{path} omits named mechanical failure {denial}"
        );
    }
}
