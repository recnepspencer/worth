//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCheckpoint {
    #[doc(hidden)]
    pub fn encode_for_test(
        native: worth_relational::facade::durability::RelationalNativeCheckpoint,
        publication: &crate::domain_computation::primary_graph::vocabulary::bootstrap::publication::WorthQueryPrimaryGraphPublication,
        accepted_outputs: &[super::super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity],
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<
        (Self, WorthQueryApplicationCheckpointSectionBytes),
        WorthQueryCheckpointCaptureDenial,
    > {
        Self::encode(native, publication, accepted_outputs, policy)
    }
}
