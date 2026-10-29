pub(crate) use {std::path::Path};
pub(crate) use {worth_foundational::canonical_milestone2_production_readiness_report, worth_foundational::canonicalization_api::canonical_public_surface_inventory, worth_foundational::CanonicalHarnessExpansionPoint};

pub(crate) fn crate_root_path(relative: &str) -> std::path::PathBuf {
    std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-foundational")).join(relative)
}