//! Production-binary execution lifecycle for authority-sealing repositories.

use super::repository::AuthoritySealingTestRepository;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

impl AuthoritySealingTestRepository {
    pub fn run_boundary_check(&self) -> (bool, String) {
        let displaced_sources = self.displace_hostile_sources_for_snapshot_seed();
        let prior_exemptions = self.install_snapshot_seed_exemptions();
        let seed = worth_harness::tooling::process_deadline::output(
            Command::new(env!("CARGO_BIN_EXE_boundary-check"))
                .arg("--root")
                .arg(&self.root)
                .arg("--config")
                .arg("tools/boundary-check/config/road1.toml")
                .arg("--update-snapshots"),
        )
        .expect("seed Phase 6 snapshots");
        self.restore_snapshot_seed_exemptions(prior_exemptions);
        self.restore_displaced_sources(displaced_sources);
        if !seed.status.success() {
            return (
                false,
                format!(
                    "snapshot seed failed:\n{}{}",
                    String::from_utf8_lossy(&seed.stdout),
                    String::from_utf8_lossy(&seed.stderr),
                ),
            );
        }
        let output = worth_harness::tooling::process_deadline::output(
            Command::new(env!("CARGO_BIN_EXE_boundary-check"))
                .arg("--root")
                .arg(&self.root)
                .arg("--config")
                .arg("tools/boundary-check/config/road1.toml"),
        )
        .expect("run boundary-check");
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        (output.status.success(), format!("{stdout}{stderr}"))
    }

    fn displace_hostile_sources_for_snapshot_seed(&self) -> Vec<(PathBuf, String)> {
        let mut displaced = Vec::new();
        for root in [
            self.root.join("cad/workspaces/worth-entry"),
            self.root.join("vendor"),
        ] {
            collect_displaced_sources(&root, &mut displaced);
        }
        for (path, source) in &displaced {
            fs::write(path, snapshot_seed_surface(path, source))
                .expect("write safe snapshot seed source");
        }
        displaced
    }

    fn restore_displaced_sources(&self, displaced: Vec<(PathBuf, String)>) {
        for (path, source) in displaced {
            fs::write(path, source).expect("restore hostile fixture source");
        }
    }

    fn install_snapshot_seed_exemptions(&self) -> Option<String> {
        let exemption_path = self
            .root
            .join("tools/boundary-check/config/generated_source_exemptions.txt");
        let prior = fs::read_to_string(&exemption_path).ok();
        let mut sources = Vec::new();
        collect_rust_source_paths(
            &self
                .root
                .join("cad/workspaces/worth-entry/crates/worth-entry-adoption/src"),
            &mut sources,
        );
        sources.sort();
        let exemptions = sources
            .iter()
            .filter_map(|path| path.strip_prefix(&self.root).ok())
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&exemption_path, exemptions).expect("write snapshot seed exemptions");
        prior
    }

    fn restore_snapshot_seed_exemptions(&self, prior: Option<String>) {
        let exemption_path = self
            .root
            .join("tools/boundary-check/config/generated_source_exemptions.txt");
        if let Some(prior) = prior {
            fs::write(exemption_path, prior).expect("restore generated-source exemptions");
        } else {
            let _ = fs::remove_file(exemption_path);
        }
    }

    pub fn cleanup(self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn collect_rust_source_paths(root: &std::path::Path, sources: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_paths(&path, sources);
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            sources.push(path);
        }
    }
}

fn snapshot_seed_surface(path: &std::path::Path, source: &str) -> String {
    if path.file_name().and_then(|name| name.to_str()) == Some("lib.rs")
        && path
            .components()
            .any(|part| part.as_os_str() == "worth-entry")
    {
        return "pub(crate) mod test_surface;\npub mod facade;\n".to_owned();
    }
    let Ok(file) = syn::parse_file(source) else {
        return "// unparseable hostile leaf displaced for snapshot seed\n".to_owned();
    };
    let mut seed = String::from("// neutral declarations preserve the snapshot facade shape\n");
    append_snapshot_seed_items(&mut seed, file.items);
    seed
}

fn append_snapshot_seed_items(seed: &mut String, items: Vec<syn::Item>) {
    for item in items {
        match item {
            syn::Item::Fn(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                seed.push_str(&format!("pub fn {}() {{}}\n", item.sig.ident));
            }
            syn::Item::Struct(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                seed.push_str(&format!("pub struct {};\n", item.ident));
            }
            syn::Item::Enum(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                seed.push_str(&format!("pub enum {} {{}}\n", item.ident));
            }
            syn::Item::Trait(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                seed.push_str(&format!("pub trait {} {{}}\n", item.ident));
            }
            syn::Item::Type(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                seed.push_str(&format!("pub type {} = ();\n", item.ident));
            }
            syn::Item::Mod(item) => {
                seed.push_str(&format!("pub mod {} {{\n", item.ident));
                if let Some((_, items)) = item.content {
                    append_snapshot_seed_items(seed, items);
                }
                seed.push_str("}\n");
            }
            _ => {}
        }
    }
}

fn collect_displaced_sources(root: &std::path::Path, displaced: &mut Vec<(PathBuf, String)>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_displaced_sources(&path, displaced);
            continue;
        }
        let source = fs::read_to_string(&path).unwrap_or_default();
        let is_hostile_leaf = path.file_name().and_then(|name| name.to_str())
            == Some("test_surface.rs")
            || path.components().any(|part| part.as_os_str() == "vendor")
            || path.file_name().and_then(|name| name.to_str()) == Some("lib.rs")
                && path
                    .components()
                    .any(|part| part.as_os_str() == "worth-entry")
            || path.file_name().and_then(|name| name.to_str()) == Some("facade.rs")
                && source.contains('*');
        if is_hostile_leaf && path.extension().and_then(|value| value.to_str()) == Some("rs") {
            displaced.push((path.clone(), source));
        }
    }
}
