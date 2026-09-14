use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;
use worth_signal::facade::branch::AdmittedSignalBranchBasis;

use super::{LoweredOwnerComponentPlan, RelationalComponentPlan, SignalComponentPlan};
use crate::branch::ProductBranchObservation;
use crate::publication::{
    NoEffectCause, NoEffectCompositePublication, ResolvedExpectedProductHead,
};

/// Lower one admitted product head into the two per-owner publication plans.
/// Branch creation is lowered elsewhere: publication has no fork route.
///
/// The matrix is total and has exactly three cells, one per component intent
/// the caller's publication stage can express:
///
/// - `RelationalOnly` (a `WithoutSignal` intent): Relational `PublishPrepared`
///   against the owner-issued candidate, Signal `RetainExact`;
/// - `SignalOnly` (a `WithSignal` intent with no Relational change): Relational
///   `RetainExact`, Signal `AdvanceExact`;
/// - `RelationalAndSignal` (a `WithSignal` intent carrying a Relational
///   change): Relational `PublishPrepared`, Signal `AdvanceExact`.
///
/// No cell retains both components, so a Retain/Retain publication is not
/// lowerable and never reaches a reservation. The admitted head carries its own
/// intent, so no second copy can disagree with it here.
pub(crate) fn lower_component_plans(
    expected: ResolvedExpectedProductHead,
    prepared_candidate: Option<PreparedRelationalCommitCandidate>,
    settled_adoption: Option<crate::publication::SettledRelationalPublicationAdoption>,
) -> Result<LoweredOwnerComponentPlan, NoEffectCompositePublication> {
    let intent = expected.intent().clone();
    let expected_head = expected.expected().clone();
    let basis = expected_head.basis();
    let relational = lower_relational_plan(
        basis.relational_basis().clone(),
        intent.changes_relational(),
        prepared_candidate,
        settled_adoption,
        &expected_head,
    )?;
    let signal = lower_signal_plan(basis.signal_basis().clone(), intent.changes_signal());
    Ok(LoweredOwnerComponentPlan::new(expected, relational, signal))
}

fn lower_relational_plan(
    expected: AdmittedRelationalBranchBasis,
    changes: bool,
    prepared_candidate: Option<PreparedRelationalCommitCandidate>,
    settled_adoption: Option<crate::publication::SettledRelationalPublicationAdoption>,
    expected_head: &ProductBranchObservation,
) -> Result<RelationalComponentPlan, NoEffectCompositePublication> {
    if !changes {
        return match (prepared_candidate, settled_adoption) {
            (None, None) => Ok(RelationalComponentPlan::retain_exact(expected)),
            (candidate, adoption) => {
                drop(candidate);
                drop(adoption);
                Err(lowering_denied(expected_head))
            }
        };
    }
    match (prepared_candidate, settled_adoption) {
        (Some(candidate), None) if candidate.branch() == expected.identity().branch_id() => Ok(
            RelationalComponentPlan::publish_prepared(expected, candidate),
        ),
        (None, Some(adoption))
            if adoption.successor_basis().identity().branch_id()
                == expected.identity().branch_id() =>
        {
            Ok(RelationalComponentPlan::adopt_settled(expected, adoption))
        }
        (candidate, adoption) => {
            drop(candidate);
            drop(adoption);
            Err(lowering_denied(expected_head))
        }
    }
}

fn lower_signal_plan(expected: AdmittedSignalBranchBasis, changes: bool) -> SignalComponentPlan {
    if changes {
        SignalComponentPlan::advance_exact(expected)
    } else {
        SignalComponentPlan::retain_exact(expected)
    }
}

fn lowering_denied(expected: &ProductBranchObservation) -> NoEffectCompositePublication {
    NoEffectCompositePublication::new(NoEffectCause::PreEffectFailure, Some(expected.clone()))
}
