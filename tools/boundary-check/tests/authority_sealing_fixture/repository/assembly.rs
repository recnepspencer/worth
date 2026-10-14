//! Base repository assembly for authority sealing proofs.

use std::fs;

use super::super::facade_projection::facade_reexports_with_external;
use super::AuthoritySealingTestRepository;

impl AuthoritySealingTestRepository {
    /// Assemble thin orchestrator + one entry-band governed crate with given lib.rs.
    pub fn assemble_with_lib_source(&self, lib_source: &str) {
        self.assemble_with_lib_source_and_config(lib_source, &self.minimal_config());
    }

    /// Assemble with an explicit road1.toml body (for substrate-admission hostiles).
    pub fn assemble_with_lib_source_and_config(&self, lib_source: &str, config: &str) {
        self.write_file(
            ".boundary-check-mock-root",
            "Declared boundary-check mock root.\n",
        );
        self.write_file("cad/docs/worthy-foundations/NAMING.md", "# naming\n");
        fs::create_dir_all(self.root.join("cad/workspaces")).expect("cad/workspaces");
        self.write_file(
            "Cargo.toml",
            r#"[workspace]
resolver = "2"
exclude = ["cad/workspaces/*"]
members = []

[workspace.metadata.worth_topology]
role = "thin_orchestrator"
road1_subworkspaces = []
forbidden_member_prefixes = ["cad/workspaces/"]
boundary_check_manifest = "tools/boundary-check/Cargo.toml"
boundary_check_config = "tools/boundary-check/config/road1.toml"
"#,
        );
        self.write_file(
            "tools/boundary-check/Cargo.toml",
            r#"[package]
name = "boundary-check"
version = "0.1.0"
edition = "2021"
publish = false
[workspace]
"#,
        );
        self.write_file(
            "tools/boundary-check/snapshots/legacy-references.toml",
            "schema_version = 1\nreferences = []\n",
        );
        self.write_file("tools/boundary-check/config/road1.toml", config);
        self.write_file(
            "tools/boundary-check/config/consumer-docs.toml",
            "documents = []\ndoctests = []\n",
        );
        self.write_file(
            "tools/boundary-check/snapshots/consumer-doc-fence-debt.toml",
            "documents = []\n",
        );
        self.write_query_audience_leaf_facades();
        self.write_worth_proof_stub();
        self.write_entry_crate(lib_source, None);
    }

    /// Assemble entry crate that path-depends on a stub substrate package.
    pub fn assemble_with_substrate_dependency(
        &self,
        lib_source: &str,
        config: &str,
        substrate_package: &str,
    ) {
        self.assemble_with_lib_source_and_config(lib_source, config);
        self.write_file(
            &format!("vendor/{substrate_package}/Cargo.toml"),
            &format!(
                r#"[package]
name = "{substrate_package}"
version = "0.1.0"
edition = "2021"
[workspace]
"#
            ),
        );
        self.write_file(
            &format!("vendor/{substrate_package}/src/lib.rs"),
            "// stub law substrate\n",
        );
        // Rewrite entry crate manifest with the substrate path dependency.
        self.write_entry_crate(lib_source, Some(substrate_package));
    }

    /// Assemble entry crate that publicly re-exports a non-governed path dependency.
    ///
    /// The dependency is placed under `vendor/` so it is not itself scanned as a
    /// governed crate — the re-export seam is the ordinary public surface under test.
    pub fn assemble_with_external_path_dependency(
        &self,
        entry_lib: &str,
        external_package: &str,
        external_lib: &str,
    ) {
        self.assemble_with_lib_source_and_config(entry_lib, &self.minimal_config());
        self.write_file(
            &format!("vendor/{external_package}/Cargo.toml"),
            &format!(
                r#"[package]
name = "{external_package}"
version = "0.1.0"
edition = "2021"
[workspace]
"#
            ),
        );
        self.write_file(
            &format!("vendor/{external_package}/src/lib.rs"),
            external_lib,
        );
        self.write_entry_crate(entry_lib, Some(external_package));
        self.write_file(
            "cad/workspaces/worth-entry/crates/worth-entry-adoption/src/facade.rs",
            &facade_reexports_with_external(entry_lib, external_package, external_lib),
        );
    }

    /// Assemble with a custom `[lib] path` and additional crate-relative source files.
    ///
    /// `crate_files` are paths relative to the entry crate root (e.g. `src/ordinary_api.rs`).
    pub fn assemble_with_lib_path_and_files(&self, lib_path: &str, crate_files: &[(&str, &str)]) {
        self.assemble_with_lib_source_and_config("// placeholder", &self.minimal_config());
        if lib_path == "src/lib.rs" {
            self.write_entry_crate_layout(lib_path, crate_files, None);
            self.write_generated_source_exemptions(&[
                "cad/workspaces/worth-entry/crates/worth-entry-adoption/src/facade.rs",
                "cad/workspaces/worth-entry/crates/worth-entry-adoption/src/test_surface.rs",
            ]);
        } else {
            let specimen = crate_files
                .iter()
                .find_map(|(path, source)| (*path == lib_path).then_some(*source))
                .expect("custom lib source");
            let mut migrated = crate_files.to_vec();
            migrated.retain(|(path, _)| *path != lib_path);
            migrated.push((lib_path, "pub(crate) mod test_surface;\npub mod facade;\n"));
            migrated.push(("src/test_surface.rs", specimen));
            self.write_entry_crate_layout(lib_path, &migrated, None);
            self.write_file(
                "cad/workspaces/worth-entry/crates/worth-entry-adoption/src/facade.rs",
                &super::super::facade_projection::facade_reexports(specimen),
            );
            self.write_generated_source_exemptions(&[
                "cad/workspaces/worth-entry/crates/worth-entry-adoption/src/lib.rs",
            ]);
        }
    }
}
