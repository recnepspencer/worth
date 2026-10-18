//! Shared ordinary Publication owner for synthetic Query audience repositories.
use std::path::Path;

/// Seed the same ordinary request-to-owning-builder shape checked in production.
pub fn write(root: &Path) {
    for (relative, source) in [
        (
            "Cargo.toml",
            "[package]\nname = \"worth-query-publication\"\nversion = \"0.1.0\"\nedition = \"2024\"\nrust-version = \"1.94\"\n[workspace]\n",
        ),
        (
            "src/lib.rs",
            "//! Synthetic Publication owner.\nmod application_entry;\n/// Publication vocabulary.\npub mod facade;\n",
        ),
        (
            "src/facade.rs",
            "//! Publication facade.\n/// Ordinary owning application entry.\npub mod application_entry {\n    pub use crate::application_entry::{WorthQueryApplicationRequest, WorthQueryOperationBuilder};\n}\n",
        ),
        (
            "src/application_entry/mod.rs",
            "//! Ordinary application entry without manual mutation transitions.\nmod request;\npub use request::{WorthQueryApplicationRequest, WorthQueryOperationBuilder};\n",
        ),
        (
            "src/application_entry/request.rs",
            r#"//! The ordinary request constructs its owning operation builder.
/// Authenticated application request shape used by the boundary contract.
pub struct WorthQueryApplicationRequest;
/// Owning operation construction shape without manual publication controls.
pub struct WorthQueryOperationBuilder;
impl WorthQueryApplicationRequest {
    /// Construct the ordinary owning operation.
    pub fn mutate(&self) -> WorthQueryOperationBuilder {
        WorthQueryOperationBuilder
    }
}
"#,
        ),
    ] {
        let path = root.join("crates/worth-query-publication").join(relative);
        std::fs::create_dir_all(path.parent().expect("fixture file parent"))
            .expect("create ordinary entry fixture directory");
        std::fs::write(path, source).expect("write ordinary entry fixture source");
    }
}
