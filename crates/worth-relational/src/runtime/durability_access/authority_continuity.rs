use crate::capabilities::RuntimeConfigSource;
use crate::durability::data::RecoveryAuthorityContinuityCheck;
use crate::runtime::RelationalRuntime;

pub(crate) fn authority_continuity_for_envelopes(
    runtime: &RelationalRuntime,
    checkpoint_envelopes: &[crate::history::data::PositionedCanonicalCommit],
    tail_log: &[crate::durability::migration::ReadmittedCanonicalCommit],
) -> RecoveryAuthorityContinuityCheck {
    let descriptor_policy = runtime
        .runtime_config()
        .schema
        .descriptor_semantics_policy
        .clone();
    let canonical_basis_policy = runtime
        .runtime_config()
        .schema
        .descriptor_canonical_basis_policy
        .clone();
    crate::durability::authority_continuity::verify(
        &descriptor_policy,
        &canonical_basis_policy,
        &runtime.services.instrumentation,
        checkpoint_envelopes
            .iter()
            .map(crate::history::data::PositionedCanonicalCommit::envelope)
            .chain(tail_log.iter().map(|entry| entry.envelope())),
    )
}
