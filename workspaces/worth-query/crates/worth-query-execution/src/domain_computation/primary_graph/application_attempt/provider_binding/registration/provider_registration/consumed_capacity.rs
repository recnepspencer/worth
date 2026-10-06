use crate::domain_computation::primary_graph::invariant_projection::{
    ConsumedOutputEvidence, ConsumedOutputVerificationStop,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::RequiredOutputDemandContext;

/// The request's meter: a managed demand's carried one, or an ordinary
/// commit's installed publication allowance. Before any application effect it
/// pays the backing the consumed edges are held in, then every check the
/// commit makes.
pub(super) fn request_meter(
    provider: &WorthQueryPrimaryGraphProvider,
    required_output_demand: Option<&mut RequiredOutputDemandContext>,
    consumed_outputs: &mut [ConsumedOutputEvidence],
) -> Result<InvalidationEditAdmission, &'static str> {
    let owner = &provider.graph.source_owner.invalidation_owner;
    let mut admission = required_output_demand.map_or_else(
        || owner.edit_admission(),
        RequiredOutputDemandContext::take_request_admission,
    );
    ConsumedOutputEvidence::admit_backing(consumed_outputs, owner, &mut admission).map_err(
        |stop| match stop {
            ConsumedOutputVerificationStop::WorkExhausted => {
                "consumed output backing exceeds request work"
            }
            ConsumedOutputVerificationStop::Unavailable
            | ConsumedOutputVerificationStop::PendingUpstream
            | ConsumedOutputVerificationStop::RetryCurrentness(_) => {
                "consumed output backing capacity unavailable"
            }
        },
    )?;
    Ok(admission)
}
