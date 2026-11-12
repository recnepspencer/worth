//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalConditionalDefinitionAdvanceBinding>("crates/worth-signal/src/branch/owner_services/conditional_execution/installation_extension/applied_binding.rs", "SignalConditionalDefinitionAdvanceBinding", 128 * 1024);
        owner.field::<Arc<SignalConditionalServiceAuthority>>("service_authority");
        owner.field::<SignalConditionalDefinitionPublicationScope>("publication_scope");
        owner.field::<AdmittedSignalBranchBasis>("predecessor");
        owner.field::<SignalBranchCellIncarnation>("incarnation");
        owner.field::<AdmittedSignalBranchBasis>("successor");
    }
}
