use super::super::*;
use std::path::PathBuf;
pub(super) const COURT: &str = "sample-court-host";
pub(super) const TARGET: &str = "sample_court_host";
pub(super) struct Fixture(pub(super) PathBuf);
impl Fixture {
    pub(super) fn new() -> Self {
        let f = Self(worth_harness::tooling::fixture_path::unique_path(
            "test-topology",
        ));
        f.write(CONFIG, "governed | sample\n");
        f.write(EXCEPTIONS, "# no exceptions\n");
        f.write(runtime_roots::CONFIG, "[[roots]]\ntype='foreign::Runtime'\nreason='Synthetic work host'\nowner='governed'\n[[roots]]\ntype='foreign::OtherRuntime'\nreason='Second synthetic work host'\n[[roots]]\ntype='ApplicationRuntime'\nreason='Owned synthetic work host'\n");
        f.manifest("autotests=false", "", "host");
        f.entry("#[test] fn lawful() {}");
        f
    }
    pub(super) fn write(&self, relative: &str, text: &str) {
        let p = self.0.join(relative);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    pub(super) fn append(&self, relative: &str, text: &str) {
        let p = self.0.join(relative);
        let old = std::fs::read_to_string(p).unwrap();
        self.write(relative, &format!("{old}\n{text}"));
    }
    pub(super) fn manifest(&self, fields: &str, targets: &str, surface: &str) {
        self.write("governed/Cargo.toml", &format!("[package]\nname='{COURT}'\nversion='0.1.0'\n{fields}\n[[test]]\nname='{TARGET}'\npath='tests/{TARGET}.rs'\n{targets}\n[package.metadata.worth.court]\nsurface='{surface}'\n"));
    }
    pub(super) fn entry(&self, source: &str) {
        self.write(&format!("governed/tests/{TARGET}.rs"), source);
    }
    pub(super) fn messages(&self) -> Vec<String> {
        validate(&self.0)
            .into_iter()
            .map(|d| d.message().to_owned())
            .collect()
    }
    pub(super) fn refuses(&self, text: &str) {
        assert!(
            self.messages().iter().any(|m| m.contains(text)),
            "{:?}",
            self.messages()
        );
    }
    pub(super) fn workspace(&self) {
        self.write(CONFIG, ". | sample\n");
        self.write("Cargo.toml", "[workspace]\nmembers=['governed','app']\n");
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let path = self.0.canonicalize().unwrap();
        let parent = self.0.parent().unwrap().canonicalize().unwrap();
        assert!(path.starts_with(&parent) && path != parent);
        std::fs::remove_dir_all(path).unwrap();
    }
}
