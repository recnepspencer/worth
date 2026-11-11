//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalConditionalDefinitionPublicationOperation>("crates/worth-signal/src/branch/owner_services/conditional_execution/installation_extension.rs", "SignalConditionalDefinitionPublicationOperation", 128 * 1024);
        owner.field::<AdmittedSignalBranchBasis>("predecessor");
        owner.field::<SignalBranchCellIncarnation>("incarnation");
        owner.field::<SignalConditionalDefinitionPublicationScope>("scope");
    }
    super::applied_binding::inline_type_budget::collect(census);
}
