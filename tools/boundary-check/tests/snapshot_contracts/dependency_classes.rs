use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

// Disabled optional edges belong to Store's existing other-team root.
// Keep the exact DAG assertions while honoring the one-build owner policy.
struct Repository {
    root: PathBuf,
}

impl Repository {
    fn new() -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("snapshot-dependency-classes-{id}"));
        write(
            &root,
            ".boundary-check-mock-root",
            "Declared boundary-check mock root.\n",
        );
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
        write(&root, "workspaces/worth-store/README.md", "fixture\n");
        write(
            &root,
            "workspaces/worth-store/Cargo.toml",
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

    fn dag_snapshot(&self) -> String {
        fs::read_to_string(
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
fn every_declared_cargo_dependency_class_is_an_exact_canonical_edge() {
    const DEPENDENCY_CLASSES: &[(&str, &str, &str)] = &[
        (
            "normal",
            "graph_alias",
            "[dependencies]\ngraph_alias={ package='worth-schema-graph', path='../worth-schema-graph' }\n",
        ),
        (
            "disabled optional",
            "optional_graph_alias",
            "[dependencies]\noptional_graph_alias={ package='worth-schema-graph', path='../worth-schema-graph', optional=true }\n",
        ),
        (
            "development",
            "dev_graph_alias",
            "[dev-dependencies]\ndev_graph_alias={ package='worth-schema-graph', path='../worth-schema-graph' }\n",
        ),
        (
            "build",
            "build_graph_alias",
            "[build-dependencies]\nbuild_graph_alias={ package='worth-schema-graph', path='../worth-schema-graph' }\n",
        ),
        (
            "target-specific disabled",
            "target_graph_alias",
            "[target.'cfg(target_os = \"none\")'.dependencies]\ntarget_graph_alias={ package='worth-schema-graph', path='../worth-schema-graph' }\n",
        ),
    ];

    for (class, alias, dependency_table) in DEPENDENCY_CLASSES {
        let repo = Repository::new();
        repo.update();
        let baseline = repo.dag_snapshot();
        write(
            &repo.root,
            CORE_MANIFEST,
            &format!("{PACKAGE}\n{dependency_table}"),
        );

        let addition = repo.run(false);
        assert!(!addition.status.success(), "accepted added {class} edge");
        assert!(
            stderr(&addition).contains("BC8003_CRATE_DAG_SNAPSHOT_DRIFT"),
            "added {class} edge produced: {}",
            stderr(&addition)
        );
        assert_eq!(baseline, repo.dag_snapshot());

        repo.update();
        let widened = repo.dag_snapshot();
        assert_ne!(baseline, widened, "{class} edge was not recorded");
        assert!(widened.contains("dependencies = [\"worth-schema-graph\"]"));
        assert!(!widened.contains(alias), "{class} alias became identity");

        write(&repo.root, CORE_MANIFEST, PACKAGE);
        let removal = repo.run(false);
        assert!(!removal.status.success(), "accepted removed {class} edge");
        assert!(
            stderr(&removal).contains("BC8003_CRATE_DAG_SNAPSHOT_DRIFT"),
            "removed {class} edge produced: {}",
            stderr(&removal)
        );
        assert_eq!(widened, repo.dag_snapshot());
    }
}

fn create_crate(root: &Path, package: &str, item: &str) {
    let base = format!("workspaces/worth-store/crates/{package}");
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

const CORE_MANIFEST: &str = "workspaces/worth-store/crates/worth-schema-core/Cargo.toml";
const PACKAGE: &str = "[package]\nname='worth-schema-core'\nversion='0.1.0'\nedition='2021'\n";
const ROOT_MANIFEST: &str = "[workspace]\nresolver='2'\nexclude=['workspaces/*']\nmembers=[]\n[workspace.metadata.worth_topology]\nrole='thin_orchestrator'\nforbidden_member_prefixes=['workspaces/']\nboundary_check_manifest='tools/boundary-check/Cargo.toml'\nboundary_check_config='tools/boundary-check/config/road1.toml'\n";
const SUBWORKSPACE_MANIFEST: &str = "[workspace]\nresolver='2'\nmembers=['crates/worth-schema-core','crates/worth-schema-graph']\n[workspace.metadata.worth_topology]\nrole='road1_subworkspace'\nconstitutional_lane='worth-store'\nmember_lane='crates/*'\nallowed_crate_prefixes=['worth-schema-']\n";
const CONFIG: &str = "root_manifest='Cargo.toml'\nforbidden_root_prefixes=['workspaces/']\nseed_skeletons=[]\n[[born_crates]]\npath='workspaces/worth-store/crates/worth-schema-core'\npackage='worth-schema-core'\n[[born_crates]]\npath='workspaces/worth-store/crates/worth-schema-graph'\npackage='worth-schema-graph'\n[machine_authority]\ncanonical_config='tools/boundary-check/config/road1.toml'\nmirrored_docs=['plans/worthy/NAMING.md']\n[naming]\nbands=['schema']\n[[naming.reserved_domains]]\ntier='worth'\nband='schema'\ndomains=['core','graph']\n[[law_substrates]]\npackage='worth-proof'\ntiers=['worth','worthy']\nbands=['schema']\n[rule_contracts]\nreplay_surfaces=[]\n[[rule_contracts.band_rules]]\nsource_band='schema'\nallowed_target_bands=['schema']\n[rule_contracts.query_audience]\nengine_package='worth-query'\naudiences=[]\n[[subworkspaces]]\npath='workspaces/worth-store'\nallowed_crate_prefixes=['worth-schema-']\nmember_lane='crates/*'\n[legacy_reference_ratchet]\ngoverned_roots=[]\nforbidden_fragments=[]\nsnapshot='tools/boundary-check/snapshots/legacy-references.toml'\nexclude_paths=[]\nreplacement_guidance='none'\n";
