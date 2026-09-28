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
        let root = std::env::temp_dir().join(format!("snapshot-contract-{id}"));
        fs::create_dir_all(&root).unwrap();
        write(&root, "Cargo.toml", ROOT_MANIFEST);
        write(&root, "plans/worthy/NAMING.md", "fixture authority\n");
        write(
            &root,
            "tools/boundary-check/Cargo.toml",
            "[package]\nname='fixture-check'\nversion='0.0.0'\n",
        );
        write(&root, "tools/boundary-check/config/road1.toml", CONFIG);
        write(
            &root,
            "tools/boundary-check/snapshots/legacy-references.toml",
            "schema_version = 1\nreferences = []\n",
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

    fn snapshot(&self, name: &str) -> Vec<u8> {
        fs::read(self.root.join("tools/boundary-check/snapshots").join(name)).unwrap()
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn regeneration_is_stable_and_ordinary_mode_never_writes() {
    let repo = Repository::new();
    repo.update();
    let first = (
        repo.snapshot("crate-dag.toml"),
        repo.snapshot("facades.toml"),
    );
    repo.update();
    assert_eq!(first.0, repo.snapshot("crate-dag.toml"));
    assert_eq!(first.1, repo.snapshot("facades.toml"));

    write(
        &repo.root,
        "cad/workspaces/worth-contracts/crates/worth-schema-core/src/facade.rs",
        "pub use crate::thing::{CoreThing, AddedThing};\n",
    );
    write(
        &repo.root,
        "cad/workspaces/worth-contracts/crates/worth-schema-core/src/thing.rs",
        "pub struct CoreThing;\npub struct AddedThing;\n",
    );
    let output = repo.run(false);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("BC8002_FACADE_SNAPSHOT_DRIFT"));
    assert_eq!(first.0, repo.snapshot("crate-dag.toml"));
    assert_eq!(first.1, repo.snapshot("facades.toml"));
}

#[test]
fn dependency_and_facade_additions_and_removals_require_regeneration() {
    let repo = Repository::new();
    repo.update();
    let core_manifest = "[package]\nname='worth-schema-core'\nversion='0.1.0'\nedition='2021'\n\n[dependencies]\ngraph_alias={ package='worth-schema-graph', path='../worth-schema-graph' }\n";
    write(
        &repo.root,
        "cad/workspaces/worth-contracts/crates/worth-schema-core/Cargo.toml",
        core_manifest,
    );
    let output = repo.run(false);
    assert!(stderr(&output).contains("BC8003_CRATE_DAG_SNAPSHOT_DRIFT"));
    repo.update();
    assert!(String::from_utf8(repo.snapshot("crate-dag.toml"))
        .unwrap()
        .contains("dependencies = [\"worth-schema-graph\"]"));

    write(
        &repo.root,
        "cad/workspaces/worth-contracts/crates/worth-schema-core/Cargo.toml",
        "[package]\nname='worth-schema-core'\nversion='0.1.0'\nedition='2021'\n",
    );
    assert!(stderr(&repo.run(false)).contains("BC8003_CRATE_DAG_SNAPSHOT_DRIFT"));
    repo.update();

    let facade_before = String::from_utf8(repo.snapshot("facades.toml")).unwrap();
    let dag_before_facade_widening = repo.snapshot("crate-dag.toml");
    write(
        &repo.root,
        "cad/workspaces/worth-contracts/crates/worth-schema-core/src/facade.rs",
        "pub use crate::thing::{CoreThing, AddedThing};\n",
    );
    write(
        &repo.root,
        "cad/workspaces/worth-contracts/crates/worth-schema-core/src/thing.rs",
        "pub struct CoreThing;\npub struct AddedThing;\n",
    );
    assert!(stderr(&repo.run(false)).contains("BC8002_FACADE_SNAPSHOT_DRIFT"));
    repo.update();
    let facade_after = String::from_utf8(repo.snapshot("facades.toml")).unwrap();
    let expected_facade_after = facade_before.replacen(
        "exports = [\"CoreThing\"]",
        "exports = [\n    \"AddedThing\",\n    \"CoreThing\",\n]",
        1,
    );
    assert_ne!(facade_before, expected_facade_after);
    assert_eq!(facade_after, expected_facade_after);
    assert_eq!(dag_before_facade_widening, repo.snapshot("crate-dag.toml"));
    write(
        &repo.root,
        "cad/workspaces/worth-contracts/crates/worth-schema-core/src/facade.rs",
        "pub use crate::thing::CoreThing;\n",
    );
    assert!(stderr(&repo.run(false)).contains("BC8002_FACADE_SNAPSHOT_DRIFT"));
}

#[test]
fn governed_non_seed_facade_denies_non_reexport_surface() {
    let repo = Repository::new();
    write(
        &repo.root,
        "tools/boundary-check/config/road1.toml",
        &CONFIG.replace(
            "[[seed_skeletons]]\npath='cad/workspaces/worth-contracts/crates/worth-schema-graph'\npackage='worth-schema-graph'\nlib_rs='src/lib.rs'\nfacade_rs='src/facade.rs'\nallowed_entries=['AGENT_CONTEXT.md','Cargo.toml','src','src/facade.rs','src/lib.rs','src/thing.rs']\n",
            "",
        ),
    );
    repo.update();

    write(
        &repo.root,
        "cad/workspaces/worth-contracts/crates/worth-schema-graph/src/facade.rs",
        "pub struct FirstSurface;\n",
    );
    let output = repo.run(false);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("facade.rs must aggregate public exports only"));
}

#[test]
fn missing_malformed_and_duplicate_baselines_fail_closed() {
    let repo = Repository::new();
    repo.update();
    fs::remove_file(
        repo.root
            .join("tools/boundary-check/snapshots/crate-dag.toml"),
    )
    .unwrap();
    fs::remove_file(
        repo.root
            .join("tools/boundary-check/snapshots/facades.toml"),
    )
    .unwrap();
    let error = stderr(&repo.run(false));
    assert!(error.contains("BC8001_SNAPSHOT_BASELINE"));
    assert!(error.contains("crate-dag.toml"));
    assert!(error.contains("facades.toml"));

    write(
        &repo.root,
        "tools/boundary-check/snapshots/crate-dag.toml",
        "not valid toml",
    );
    write(
        &repo.root,
        "tools/boundary-check/snapshots/facades.toml",
        "schema_version=1\n[[facades]]\npackage='x'\nexports=['A','A']\n",
    );
    let error = stderr(&repo.run(false));
    assert!(error.contains("parse"));
    assert!(error.contains("duplicate value A"));
}

#[test]
fn illegal_dependency_cannot_be_blessed() {
    let repo = Repository::new();
    repo.update();
    write(
        &repo.root,
        "tools/boundary-check/config/road1.toml",
        &CONFIG.replace("allowed_target_bands=['schema']", "allowed_target_bands=[]"),
    );
    write(
        &repo.root,
        "cad/workspaces/worth-contracts/crates/worth-schema-core/Cargo.toml",
        "[package]\nname='worth-schema-core'\nversion='0.1.0'\nedition='2021'\n\n[dependencies]\ngraph_alias={ package='worth-schema-graph', path='../worth-schema-graph' }\n",
    );
    let before = repo.snapshot("crate-dag.toml");
    let output = repo.run(true);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("BC2001_BAND_DEPENDENCY_VIOLATION"));
    assert_eq!(before, repo.snapshot("crate-dag.toml"));
}

fn create_crate(root: &Path, package: &str, item: &str) {
    let base = format!("cad/workspaces/worth-contracts/crates/{package}");
    write(
        root,
        &format!("{base}/Cargo.toml"),
        &format!("[package]\nname='{package}'\nversion='0.1.0'\nedition='2021'\n"),
    );
    write(root, &format!("{base}/AGENT_CONTEXT.md"), "generated\n");
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

const ROOT_MANIFEST: &str = r#"[workspace]
resolver='2'
exclude=['cad/workspaces/*']
members=[]
[workspace.metadata.worth_topology]
role='thin_orchestrator'
forbidden_member_prefixes=['cad/workspaces/']
boundary_check_manifest='tools/boundary-check/Cargo.toml'
boundary_check_config='tools/boundary-check/config/road1.toml'
"#;

const SUBWORKSPACE_MANIFEST: &str = r#"[workspace]
resolver='2'
members=['crates/worth-schema-core','crates/worth-schema-graph']
[workspace.metadata.worth_topology]
role='road1_subworkspace'
constitutional_lane='worth-contracts'
member_lane='crates/*'
allowed_crate_prefixes=['worth-schema-']
"#;

const CONFIG: &str = r#"root_manifest='Cargo.toml'
forbidden_root_prefixes=['cad/workspaces/']
[machine_authority]
canonical_config='tools/boundary-check/config/road1.toml'
mirrored_docs=['plans/worthy/NAMING.md']
[naming]
bands=['schema']
[[naming.reserved_domains]]
tier='worth'
band='schema'
domains=['core','graph']
[[law_substrates]]
package='worth-proof'
tiers=['worth','worthy']
bands=['schema']
[rule_contracts]
replay_surfaces=[]
[[rule_contracts.band_rules]]
source_band='schema'
allowed_target_bands=['schema']
[rule_contracts.query_audience]
engine_package='worth-query'
audiences=[]
[[born_crates]]
path='cad/workspaces/worth-contracts/crates/worth-schema-core'
package='worth-schema-core'
[[born_crates]]
path='cad/workspaces/worth-contracts/crates/worth-schema-graph'
package='worth-schema-graph'
[[seed_skeletons]]
path='cad/workspaces/worth-contracts/crates/worth-schema-core'
package='worth-schema-core'
lib_rs='src/lib.rs'
facade_rs='src/facade.rs'
allowed_entries=['AGENT_CONTEXT.md','Cargo.toml','src','src/facade.rs','src/lib.rs','src/thing.rs']
[[seed_skeletons]]
path='cad/workspaces/worth-contracts/crates/worth-schema-graph'
package='worth-schema-graph'
lib_rs='src/lib.rs'
facade_rs='src/facade.rs'
allowed_entries=['AGENT_CONTEXT.md','Cargo.toml','src','src/facade.rs','src/lib.rs','src/thing.rs']
[[subworkspaces]]
path='cad/workspaces/worth-contracts'
allowed_crate_prefixes=['worth-schema-']
member_lane='crates/*'
[legacy_reference_ratchet]
governed_roots=[]
forbidden_fragments=[]
snapshot='tools/boundary-check/snapshots/legacy-references.toml'
exclude_paths=[]
replacement_guidance='none'
"#;
