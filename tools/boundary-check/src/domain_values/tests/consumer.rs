//! Real metadata-only consumer input, followed by the ordinary BC9019 guard.
use crate::{config::Code, domain_values, inputs::Inputs};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

const REGISTRY: &str = "domain-values.toml";
const BINDING: &str = include_str!("../../../fixtures/consumer/src/lib.rs.in");
const VALUES: &str = include_str!("../../../fixtures/consumer/src/values.rs.in");
const REGISTRATION: &str = include_str!("../../../fixtures/consumer/domain-values.toml");

struct Consumer(PathBuf);
impl Consumer {
    fn new(values: &str, registry: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "worth-domain-consumer-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("src")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            include_str!("../../../fixtures/consumer/Cargo.toml"),
        )
        .unwrap();
        fs::write(root.join("src/lib.rs"), BINDING).unwrap();
        fs::write(root.join("src/values.rs"), values).unwrap();
        fs::write(root.join(REGISTRY), registry).unwrap();
        let init = Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            init.status.success(),
            "{}",
            String::from_utf8_lossy(&init.stderr)
        );
        Self(root.canonicalize().unwrap())
    }

    fn diagnostics(&self) -> Vec<crate::config::Diagnostic> {
        let input = Inputs::read_domain_sources(&self.0, REGISTRY).unwrap();
        assert!(input.graphs["Cargo.toml"].resolve.is_none());
        assert!(!self.0.join("target").exists(), "consumer was compiled");
        domain_values::check_registry(&input, REGISTRY)
    }
}
impl Drop for Consumer {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn registered_reference_free_consumer_passes_without_compilation() {
    assert!(Consumer::new(VALUES, REGISTRATION).diagnostics().is_empty());
}

#[test]
fn unregistered_marked_consumer_binding_reports_its_value_path() {
    let diagnostics = Consumer::new(VALUES, "binding=[]").diagnostics();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, Code::DomainValue);
    assert!(diagnostics[0]
        .message
        .contains("src/lib.rs::MeasurementBinding"));
    assert!(diagnostics[0].message.contains("values :: Measurement"));
    assert!(diagnostics[0].message.contains("needs exactly one"));
}

#[test]
fn registered_consumer_cannot_contain_an_owner_forbidden_name() {
    let values = "pub struct CellName(u64); pub struct Measurement { name: CellName }";
    let diagnostics = Consumer::new(values, REGISTRATION).diagnostics();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, Code::DomainValue);
    assert!(diagnostics[0].message.contains("MeasurementBinding"));
    assert!(diagnostics[0]
        .message
        .contains("reference/key/name forbidden"));
    assert!(diagnostics[0].message.contains("CellName"));
}

#[test]
fn a_nested_consumer_struct_cannot_hide_an_owner_forbidden_name() {
    let values = "pub struct CellName(u64); struct Nested { name: CellName } pub struct Measurement { nested: Option<Box<Nested>> }";
    let diagnostics = Consumer::new(values, REGISTRATION).diagnostics();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, Code::DomainValue);
    assert!(diagnostics[0].message.contains("MeasurementBinding"));
    assert!(diagnostics[0]
        .message
        .contains("reference/key/name forbidden"));
    assert!(diagnostics[0].message.contains("CellName"));
}
