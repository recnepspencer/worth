use worth_relational::facade::mvcc::RelationalOperationInterruption;

use crate::domain_computation::primary_graph::invariant_projection::{
    ConsumedOutputEvidence, ConsumedOutputVerificationStop,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::RequiredOutputDemandContext;

/// Why an application attempt did not register. The request's cancellation
/// or elapsed deadline is its own cause; every other refusal rejects the
/// provider's plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum ApplicationAttemptRegistrationStop {
    Rejected(&'static str),
    Interrupted(RelationalOperationInterruption),
}

impl From<&'static str> for ApplicationAttemptRegistrationStop {
    fn from(reason: &'static str) -> Self {
        Self::Rejected(reason)
    }
}

/// The request's meter: a managed demand's carried one, or an ordinary
/// commit's installed publication allowance. Before any application effect it
/// pays the backing the consumed edges are held in, then every check the
/// commit makes.
pub(super) fn request_meter(
    provider: &WorthQueryPrimaryGraphProvider,
    required_output_demand: Option<&mut RequiredOutputDemandContext>,
    consumed_outputs: &mut [ConsumedOutputEvidence],
) -> Result<InvalidationEditAdmission, ApplicationAttemptRegistrationStop> {
    let owner = &provider.graph.source_owner.invalidation_owner;
    let mut admission = required_output_demand.map_or_else(
        || owner.edit_admission(),
        RequiredOutputDemandContext::take_request_admission,
    );
    ConsumedOutputEvidence::admit_backing(consumed_outputs, owner, &mut admission)
        .map_err(backing_stop)?;
    Ok(admission)
}

fn backing_stop(stop: ConsumedOutputVerificationStop) -> ApplicationAttemptRegistrationStop {
    match stop {
        ConsumedOutputVerificationStop::Interrupted(event) => {
            ApplicationAttemptRegistrationStop::Interrupted(event.interruption())
        }
        ConsumedOutputVerificationStop::WorkExhausted => {
            "consumed output backing exceeds request work".into()
        }
        ConsumedOutputVerificationStop::CapacityExhausted
        | ConsumedOutputVerificationStop::Unavailable
        | ConsumedOutputVerificationStop::PendingUpstream
        | ConsumedOutputVerificationStop::RetryCurrentness(_) => {
            "consumed output backing capacity unavailable".into()
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_relational::facade::mvcc::{
        RelationalCancellationSource, RelationalInterruptionBoundary, RelationalOperationControl,
    };

    use super::*;

    /// An interrupted backing admission is the request's interruption, so
    /// the attempt ends Cancelled or TimedOut, not as a rejected plan.
    #[test]
    fn an_interrupted_backing_is_the_interruption_not_a_rejection() {
        let source = RelationalCancellationSource::new();
        source.cancel();
        let event = RelationalOperationControl::from(source.token())
            .observe(RelationalInterruptionBoundary::PublicationPreflight)
            .expect("a cancelled control is interrupted");
        assert_eq!(
            backing_stop(ConsumedOutputVerificationStop::Interrupted(event)),
            ApplicationAttemptRegistrationStop::Interrupted(
                RelationalOperationInterruption::Cancelled
            )
        );
    }
}
