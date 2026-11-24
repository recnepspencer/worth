pub mod artifact_terms;
pub mod descriptor_terms;
pub mod mutation_program_terms;
pub mod portable_aspect_patch_terms;
pub mod primitive_terms;

pub use artifact_terms::{
    commit_validation_summary_digest, lowering_summary_digest, native_entity_fields_scope_digest,
    native_entity_replacement_scope_digest, preview_validation_cost_digest,
    runtime_invariant_catalog_digest, serial_intent_scope_digest,
};
pub use descriptor_terms::{commit_strategy_descriptor_digest, commit_strategy_registry_digest};

use primitive_terms::StrategyDigestBytes;

fn commit_strategy_digest(
    domain: &'static str,
    fill: impl FnOnce(&mut StrategyDigestBytes),
) -> [u8; 32] {
    StrategyDigestBytes::digest(domain, fill)
}

fn commit_strategy_hex_digest(
    domain: &'static str,
    fill: impl FnOnce(&mut StrategyDigestBytes),
) -> String {
    StrategyDigestBytes::hex_digest(domain, fill)
}
