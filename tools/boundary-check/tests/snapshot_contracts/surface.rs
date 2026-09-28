use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct Repository {
    root: PathBuf,
}

impl Repository {
    fn new() -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("snapshot-surface-{id}"));
        write(&root, "Cargo.toml", ROOT_MANIFEST);
        write(&root, "plans/worthy/NAMING.md", "fixture\n");
        write(
            &root,
            "tools/boundary-check/Cargo.toml",
            "[package]\nname='fixture-check'\nversion='0.0.0'\n",
        );
        write(&root, "tools/boundary-check/config/road1.toml", CONFIG);
        write(
            &root,
            "tools/boundary-check/snapshots/legacy-references.toml",
            "schema_version=1\nreferences=[]\n",
        );
        write(
            &root,
            "cad/workspaces/worth-contracts/README.md",
            "fixture\n",
        );
        write(
            &root,
            "cad/workspaces/worth-contracts/Cargo.toml",
            SUBWORKSPACE_MANIFEST,
        );
        create_crate(&root, "worth-schema-core", "CoreThing");
        create_crate(&root, "worth-schema-graph", "GraphThing");
        Self { root }
    }

    fn run(&self, update: bool) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_boundary-check"));
        command
            .arg("--root")
            .arg(&self.root)
            .arg("--config")
            .arg("tools/boundary-check/config/road1.toml");
        if update {
            command.arg("--update-snapshots");
        }
        command.output().unwrap()
    }

    fn update(&self) {
        let output = self.run(true);
        assert!(output.status.success(), "{}", stderr(&output));
    }
    fn facade_snapshot(&self) -> String {
        fs::read_to_string(
            self.root
                .join("tools/boundary-check/snapshots/facades.toml"),
        )
        .unwrap()
    }

    fn dag_snapshot(&self) -> Vec<u8> {
        fs::read(
            self.root
                .join("tools/boundary-check/snapshots/crate-dag.toml"),
        )
        .unwrap()
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn governed_surface_cannot_bypass_facade_through_lib_or_exported_macro() {
    let repo = Repository::new();
    repo.update();
    let lib = "cad/workspaces/worth-contracts/crates/worth-schema-core/src/lib.rs";
    write(
        &repo.root,
        lib,
        "mod thing;\npub mod facade;\npub use crate::thing::CoreThing;\n",
    );
    assert!(stderr(&repo.run(false)).contains("may expose only exactly-bound"));

    write(&repo.root, lib, "mod thing;\npub mod facade;\n");
    write(
        &repo.root,
        "cad/workspaces/worth-contracts/crates/worth-schema-core/src/thing.rs",
        "pub struct CoreThing;\n#[macro_export]\nmacro_rules! second_surface { () => {} }\n",
    );
    assert!(stderr(&repo.run(true)).contains("must not bypass facade.rs"));
}

#[test]
fn every_public_root_item_kind_is_rejected_before_snapshots_can_bless_it() {
    const PUBLIC_ITEMS: &[(&str, &str)] = &[
        ("const", "pub const BYPASS: usize = 0;"),
        ("enum", "pub enum Bypass { Variant }"),
        ("extern crate", "pub extern crate graph_alias;"),
        ("fn", "pub fn bypass() {}"),
        ("mod", "pub mod bypass {}"),
        ("static", "pub static BYPASS: usize = 0;"),
        ("struct", "pub struct Bypass;"),
        ("trait", "pub trait Bypass {}"),
        ("trait alias", "pub trait Bypass = Send;"),
        ("type", "pub type Bypass = usize;"),
        ("union", "pub union Bypass { value: usize }"),
        ("use", "pub use crate::thing::CoreThing;"),
    ];

    for (item_kind, public_item) in PUBLIC_ITEMS {
        let repo = Repository::new();
        repo.update();
        let facade_before = repo.facade_snapshot();
        let dag_before = repo.dag_snapshot();
        write(
            &repo.root,
            "cad/workspaces/worth-contracts/crates/worth-schema-core/src/lib.rs",
            &format!("mod thing;\npub mod facade;\n{public_item}\n"),
        );

        for update in [false, true] {
            let output = repo.run(update);
            assert!(!output.status.success(), "accepted public {item_kind}");
            let denial = stderr(&output);
            let expected_denial = if *item_kind == "extern crate" {
                "BC7001_AUTHORITY_SEALING"
            } else {
                "may expose only exactly-bound"
            };
            assert!(
                denial.contains(expected_denial),
                "public {item_kind} produced: {}",
                denial
            );
            assert_eq!(facade_before, repo.facade_snapshot());
            assert_eq!(dag_before, repo.dag_snapshot());
        }
    }
}

#[test]
fn facade_module_must_bind_the_canonical_out_of_line_file() {
    let repo = Repository::new();
    repo.update();
    let lib = "cad/workspaces/worth-contracts/crates/worth-schema-core/src/lib.rs";
    let alternate =
        "cad/workspaces/worth-contracts/crates/worth-schema-core/src/alternate_facade.rs";
    write(&repo.root, alternate, "pub use crate::thing::CoreThing;\n");

    for source in [
        "mod thing;\n#[path = \"alternate_facade.rs\"]\npub mod facade;\n",
        "mod thing;\npub mod facade { pub use crate::thing::CoreThing; }\n",
        "mod thing;\nmod facade;\n",
        "mod thing;\n",
    ] {
        write(&repo.root, lib, source);
        let check = repo.run(false);
        assert!(!check.status.success(), "check accepted {source}");
        let update = repo.run(true);
        assert!(!update.status.success(), "update accepted {source}");
    }
}

#[test]
fn custom_library_path_binds_snapshot_and_macro_checks_to_its_sibling_facade() {
    let repo = Repository::new();
    repo.update();
    let base = "cad/workspaces/worth-contracts/crates/worth-schema-core";
    write(
        &repo.root,
        &format!("{base}/Cargo.toml"),
        "[package]\nname='worth-schema-core'\nversion='0.1.0'\nedition='2021'\n[lib]\npath='compiled/root.rs'\n",
    );
    fs::remove_dir_all(repo.root.join(format!("{base}/src")))
        .expect("remove retired default library source");
    write(
        &repo.root,
        &format!("{base}/compiled/root.rs"),
        "mod thing;\npub mod facade;\n",
    );
    write(
        &repo.root,
        &format!("{base}/compiled/thing.rs"),
        "pub struct CompiledThing;\n",
    );
    write(
        &repo.root,
        &format!("{base}/compiled/facade.rs"),
        "pub use crate::thing::CompiledThing;\n",
    );

    let output = repo.run(false);
    assert!(
        stderr(&output).contains("BC8002_FACADE_SNAPSHOT_DRIFT"),
        "{}",
        stderr(&output)
    );
    repo.update();
    let snapshot = repo.facade_snapshot();
    assert!(snapshot.contains("CompiledThing"));
    assert!(!snapshot.contains("CoreThing"));

    write(
        &repo.root,
        &format!("{base}/compiled/thing.rs"),
        "pub struct CompiledThing;\n#[macro_export]\nmacro_rules! compiled_bypass { () => {} }\n",
    );
    assert!(stderr(&repo.run(true)).contains("must not bypass facade.rs"));
}

#[test]
fn exported_macro_in_out_of_directory_path_module_is_denied() {
    let repo = Repository::new();
    repo.update();
    let base = "cad/workspaces/worth-contracts/crates/worth-schema-core";
    write(
        &repo.root,
        &format!("{base}/src/lib.rs"),
        "#[path = \"../private_surface.rs\"]\nmod private_surface;\nmod thing;\npub mod facade;\n",
    );
    write(
        &repo.root,
        &format!("{base}/private_surface.rs"),
        "#[macro_export]\nmacro_rules! escaped_surface { () => {} }\n",
    );

    for update in [false, true] {
        let output = repo.run(update);
        assert!(!output.status.success());
        assert!(stderr(&output).contains("must not bypass facade.rs"));
    }
}

#[test]
fn named_reexport_aliases_track_addition_removal_and_rename() {
    let repo = Repository::new();
    repo.update();
    let facade = "cad/workspaces/worth-contracts/crates/worth-schema-core/src/facade.rs";
    write(
        &repo.root,
        facade,
        "pub use crate::thing::{CoreThing, CoreThing as graph_alias};\n",
    );
    let output = repo.run(false);
    assert!(
        stderr(&output).contains("BC8002_FACADE_SNAPSHOT_DRIFT"),
        "{}",
        stderr(&output)
    );
    repo.update();
    let snapshot = repo.facade_snapshot();
    assert!(snapshot.contains("\"CoreThing\"") && snapshot.contains("\"graph_alias\""));

    write(&repo.root, facade, "pub use crate::thing::CoreThing;\n");
    let output = repo.run(false);
    assert!(
        stderr(&output).contains("BC8002_FACADE_SNAPSHOT_DRIFT"),
        "{}",
        stderr(&output)
    );
    write(
        &repo.root,
        facade,
        "pub use crate::thing::{CoreThing, CoreThing as second_alias};\n",
    );
    assert!(stderr(&repo.run(false)).contains("BC8002_FACADE_SNAPSHOT_DRIFT"));
}

#[test]
fn crate_lane_manifest_must_be_a_workspace_member() {
    let repo = Repository::new();
    create_crate(&repo.root, "worth-schema-shadow", "ShadowThing");
    let output = repo.run(true);
    assert!(
        stderr(&output).contains("must be admitted by workspace.members"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn conditional_facade_reexport_cannot_bypass_exact_snapshot() {
    let repo = Repository::new();
    repo.update();
    let facade = "cad/workspaces/worth-contracts/crates/worth-schema-core/src/facade.rs";
    write(
        &repo.root,
        facade,
        "#[cfg(feature = \"wide\")]\npub use crate::thing::CoreThing;\n",
    );
    for update in [false, true] {
        let output = repo.run(update);
        assert!(!output.status.success());
        assert!(stderr(&output).contains("cannot form an exact compiled surface"));
    }
}

fn create_crate(root: &Path, package: &str, item: &str) {
    let base = format!("cad/workspaces/worth-contracts/crates/{package}");
    write(
        root,
        &format!("{base}/Cargo.toml"),
        &format!("[package]\nname='{package}'\nversion='0.1.0'\nedition='2021'\n"),
    );
    write(
        root,
        &format!("{base}/src/lib.rs"),
        "mod thing;\npub mod facade;\n",
    );
    write(
        root,
        &format!("{base}/src/facade.rs"),
        &format!("pub use crate::thing::{item};\n"),
    );
    write(
        root,
        &format!("{base}/src/thing.rs"),
        &format!("pub struct {item};\n"),
    );
}

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

const ROOT_MANIFEST: &str = "[workspace]\nresolver='2'\nexclude=['cad/workspaces/*']\nmembers=[]\n[workspace.metadata.worth_topology]\nrole='thin_orchestrator'\nforbidden_member_prefixes=['cad/workspaces/']\nboundary_check_manifest='tools/boundary-check/Cargo.toml'\nboundary_check_config='tools/boundary-check/config/road1.toml'\n";
const SUBWORKSPACE_MANIFEST: &str = "[workspace]\nresolver='2'\nmembers=['crates/worth-schema-core','crates/worth-schema-graph']\n[workspace.metadata.worth_topology]\nrole='road1_subworkspace'\nconstitutional_lane='worth-contracts'\nmember_lane='crates/*'\nallowed_crate_prefixes=['worth-schema-']\n";
const CONFIG: &str = "root_manifest='Cargo.toml'\nforbidden_root_prefixes=['cad/workspaces/']\nseed_skeletons=[]\n[[born_crates]]\npath='cad/workspaces/worth-contracts/crates/worth-schema-core'\npackage='worth-schema-core'\n[[born_crates]]\npath='cad/workspaces/worth-contracts/crates/worth-schema-graph'\npackage='worth-schema-graph'\n[machine_authority]\ncanonical_config='tools/boundary-check/config/road1.toml'\nmirrored_docs=['plans/worthy/NAMING.md']\n[naming]\nbands=['schema']\n[[naming.reserved_domains]]\ntier='worth'\nband='schema'\ndomains=['core','graph']\n[[law_substrates]]\npackage='worth-proof'\ntiers=['worth','worthy']\nbands=['schema']\n[rule_contracts]\nreplay_surfaces=[]\n[[rule_contracts.band_rules]]\nsource_band='schema'\nallowed_target_bands=['schema']\n[rule_contracts.query_audience]\nengine_package='worth-query'\naudiences=[]\n[[subworkspaces]]\npath='cad/workspaces/worth-contracts'\nallowed_crate_prefixes=['worth-schema-']\nmember_lane='crates/*'\n[legacy_reference_ratchet]\ngoverned_roots=[]\nforbidden_fragments=[]\nsnapshot='tools/boundary-check/snapshots/legacy-references.toml'\nexclude_paths=[]\nreplacement_guidance='none'\n";
