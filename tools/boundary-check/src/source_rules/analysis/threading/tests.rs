use super::enforce_threading_boundary;
use crate::config::ThreadingSiteConfig;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "worth-threading-boundary-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn crate_at(&self, relative: &str, manifest_tail: &str, source: &str) {
        let crate_root = self.0.join(relative);
        fs::create_dir_all(crate_root.join("src")).unwrap();
        fs::write(crate_root.join("Cargo.toml"), format!(
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n{manifest_tail}"
        )).unwrap();
        fs::write(crate_root.join("src/lib.rs"), source).unwrap();
    }

    fn root(&self) -> &Path {
        &self.0
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn declaration(
    source: &str,
    item: &str,
    kind: &str,
    category: &str,
    phase: Option<u8>,
) -> ThreadingSiteConfig {
    ThreadingSiteConfig {
        source: source.into(),
        item: item.into(),
        kind: kind.into(),
        count: 1,
        category: category.into(),
        reason: "the named site has this lifecycle".into(),
        retire_phase: phase,
    }
}

#[test]
fn scans_root_crates_and_workspace_apps_but_not_test_only_items() {
    let fixture = Fixture::new();
    fixture.crate_at(
        "crates/root",
        "",
        r#"
        use std::thread::{self as worker, Builder as Launch};
        fn run() {
            worker::spawn(|| {});
            std::thread::scope(|scope| { scope.spawn(|| {}); });
            Launch::new().spawn(|| {}).unwrap();
        }
        macro_rules! hidden { () => { std::thread::spawn(|| {}); } }
        #[cfg(test)] mod tests;
        #[cfg(test)] fn test_thread() { std::thread::spawn(|| {}); }
        #[cfg(feature = "optional")] fn feature_thread() { std::thread::spawn(|| {}); }
    "#,
    );
    fixture.crate_at(
        "workspaces/application/apps/app",
        "[dependencies]\nrayon = \"1\"\n",
        r#"
        use rayon::prelude::*;
        fn work() { let _ = [1, 2].par_iter(); }
    "#,
    );
    fixture.crate_at(
        "workspaces/application/tools/runner",
        "",
        "fn run() { std::thread::spawn(|| {}); }",
    );
    fs::write(
        fixture.root().join("crates/root/src/tests.rs"),
        "fn test_thread() { std::thread::spawn(|| {}); }",
    )
    .unwrap();
    let diagnostics = enforce_threading_boundary(fixture.root(), &[]);
    let output = format!("{diagnostics:?}");
    for needle in [
        "crates/root/src/lib.rs",
        "thread-spawn",
        "thread-scope",
        "thread-builder",
        "feature_thread",
        "workspaces/application/apps/app/src/lib.rs",
        "rayon-call",
        "rayon-import",
        "rayon-dependency",
        "workspaces/application/tools/runner/src/lib.rs",
    ] {
        assert!(output.contains(needle), "missing {needle}: {output}");
    }
    assert!(
        !output.contains("test_thread"),
        "test-only code leaked into production: {output}"
    );
}

#[test]
fn exact_declarations_accept_owner_threads_and_reject_growth_or_stale_sites() {
    let fixture = Fixture::new();
    fixture.crate_at(
        "crates/owner",
        "",
        "fn serve() { std::thread::spawn(|| {}); }",
    );
    let site = declaration(
        "crates/owner/src/lib.rs",
        "serve",
        "thread-spawn",
        "owner-thread",
        None,
    );
    assert!(enforce_threading_boundary(fixture.root(), &[site.clone()]).is_empty());

    fs::write(
        fixture.root().join("crates/owner/src/lib.rs"),
        "fn serve() { std::thread::spawn(|| {}); std::thread::spawn(|| {}); }",
    )
    .unwrap();
    let grown = format!(
        "{:?}",
        enforce_threading_boundary(fixture.root(), &[site.clone()])
    );
    assert!(
        grown.contains("expects 1 production site(s), found 2"),
        "{grown}"
    );

    fs::write(
        fixture.root().join("crates/owner/src/lib.rs"),
        "fn serve() {}",
    )
    .unwrap();
    let stale = format!("{:?}", enforce_threading_boundary(fixture.root(), &[site]));
    assert!(
        stale.contains("expects 1 production site(s), found 0"),
        "{stale}"
    );
}

#[test]
fn aliases_and_macro_bodies_cannot_hide_new_workers() {
    let fixture = Fixture::new();
    fixture.crate_at(
        "crates/aliases",
        "",
        r#"
        use std::thread::{self as worker, Builder as Launch};
        fn dispatch() { worker::spawn(|| {}); Launch::new().spawn(|| {}).unwrap(); }
        macro_rules! hidden { () => { std::thread::scope(|scope| scope.spawn(|| {})) } }
    "#,
    );
    let output = format!("{:?}", enforce_threading_boundary(fixture.root(), &[]));
    assert!(output.contains("`dispatch` / `thread-spawn`"), "{output}");
    assert!(output.contains("`dispatch` / `thread-builder`"), "{output}");
    assert!(output.contains("`` / `thread-scope`"), "{output}");
}

#[test]
fn aliases_of_std_and_manifest_targets_outside_src_are_scanned() {
    let fixture = Fixture::new();
    fixture.crate_at(
        "crates/outside",
        "[lib]\npath = \"library.rs\"\n[[bin]]\nname = \"outside-bin\"\npath = \"application/main.rs\"\n[[test]]\nname = \"excluded-test\"\npath = \"suite.rs\"\n",
        "fn ignored_unreachable_source() {}",
    );
    fs::create_dir_all(fixture.root().join("crates/outside/application")).unwrap();
    fs::write(
        fixture.root().join("crates/outside/library.rs"),
        "use std as platform; fn run() { platform::thread::spawn(|| {}); }",
    )
    .unwrap();
    fs::write(
        fixture.root().join("crates/outside/application/main.rs"),
        "fn main() { std::thread::scope(|scope| { scope.spawn(|| {}); }); }",
    )
    .unwrap();
    fs::write(
        fixture.root().join("crates/outside/build.rs"),
        "fn main() { std::thread::Builder::new(); }",
    )
    .unwrap();
    fs::write(
        fixture.root().join("crates/outside/suite.rs"),
        "fn test_only() { std::thread::spawn(|| {}); }",
    )
    .unwrap();
    let output = format!("{:?}", enforce_threading_boundary(fixture.root(), &[]));
    for needle in [
        "crates/outside/library.rs",
        "`run` / `thread-spawn`",
        "crates/outside/application/main.rs",
        "`main` / `thread-scope`",
        "crates/outside/build.rs",
        "`main` / `thread-builder`",
    ] {
        assert!(output.contains(needle), "missing {needle}: {output}");
    }
    assert!(!output.contains("suite.rs"), "test target leaked: {output}");
}

#[test]
fn legacy_parallel_requires_a_retirement_phase_and_cannot_disguise_rayon() {
    let fixture = Fixture::new();
    fixture.crate_at(
        "crates/compute",
        "[dependencies]\nrayon = \"1\"\n",
        "fn work() {}",
    );
    let source = "crates/compute/Cargo.toml";
    let valid = declaration(
        source,
        "dependencies",
        "rayon-dependency",
        "legacy-parallel",
        Some(3),
    );
    assert!(enforce_threading_boundary(fixture.root(), &[valid.clone()]).is_empty());
    let mut wrong = valid.clone();
    wrong.category = "owner-thread".into();
    assert!(
        format!("{:?}", enforce_threading_boundary(fixture.root(), &[wrong]))
            .contains("Rayon belongs to the legacy parallel ratchet")
    );
    let mut missing_phase = valid;
    missing_phase.retire_phase = None;
    assert!(format!(
        "{:?}",
        enforce_threading_boundary(fixture.root(), &[missing_phase])
    )
    .contains("retirement phase"));

    fs::write(fixture.root().join(source),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[dependencies]\nrayon = \"1\"\nworkers = { package = \"rayon\", version = \"1\" }\n").unwrap();
    let grown = format!(
        "{:?}",
        enforce_threading_boundary(
            fixture.root(),
            &[declaration(
                source,
                "dependencies",
                "rayon-dependency",
                "legacy-parallel",
                Some(3)
            )]
        )
    );
    assert!(
        grown.contains("expects 1 production site(s), found 2"),
        "{grown}"
    );
}
