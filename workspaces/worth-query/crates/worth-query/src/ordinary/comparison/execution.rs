use super::context::WorthQueryComparisonPairAuthority;
use super::diff::assemble_query_shaped_row_changes;
use super::{
    WorthQueryComparisonBasisEvidence, WorthQueryComparisonBasisFamily,
    WorthQueryComparisonBasisPairEvidence, WorthQueryComparisonChange,
    WorthQueryComparisonCompletion, WorthQueryComparisonCorrespondence,
    WorthQueryComparisonCorrespondencePosture, WorthQueryComparisonJourneyCounters,
    WorthQueryComparisonMaterialization, WorthQueryComparisonNextAction,
    WorthQueryComparisonOutcome, WorthQueryComparisonRequest, WorthQueryComparisonStop,
    WorthQueryComparisonStopSource,
};
use crate::correspondence::{
    resolve_correspondence_evidence, CorrespondenceEvaluationRequest,
    StructuralCandidateDiscoveryPlan, StructuralCandidateOrderingContract,
};
use crate::ordinary::history::WorthQueryHistoricalPathDeclaration;
use crate::ordinary::read::current;
use crate::runtime::{WorthQueryReadResult, WorthQueryWorkspace};

mod sealed {
    use crate::runtime::WorthQueryWorkspace;

    pub trait Sealed {}

    impl Sealed for &mut WorthQueryWorkspace {}
    impl Sealed for (&mut WorthQueryWorkspace, &mut WorthQueryWorkspace) {}
}

/// The sealed execution resource accepted by a comparison request.
///
/// A current-to-retained context accepts one workspace. A branch-to-branch
/// context accepts a pair. In both cases the sealed context, not the resource
/// argument, owns basis authority and is revalidated before execution.
pub trait WorthQueryComparisonExecution: sealed::Sealed {
    #[doc(hidden)]
    fn execute(
        self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: WorthQueryComparisonRequest,
    ) -> WorthQueryComparisonOutcome;
}

impl WorthQueryComparisonRequest {
    pub fn run(
        self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        execution: impl WorthQueryComparisonExecution,
    ) -> WorthQueryComparisonOutcome {
        execution.execute(request_execution, self)
    }
}

impl WorthQueryComparisonExecution for &mut WorthQueryWorkspace {
    fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: WorthQueryComparisonRequest,
    ) -> WorthQueryComparisonOutcome {
        let WorthQueryComparisonPairAuthority::CurrentAndRetained { current, retained } =
            request.context.authority.clone()
        else {
            return invalid_execution_resource();
        };
        let counters = WorthQueryComparisonJourneyCounters::validate_pair();
        if !current.matches(execution, self)
            || !retained.admits_snapshot(&self.snapshot_identity_with_request(execution))
        {
            return stale_pair(counters);
        }

        let counters = counters.execute_left();
        let left = match execute_current_read(execution, request.declaration.left_read, self) {
            Ok(result) => result,
            Err(reason) => {
                return stopped(
                    WorthQueryComparisonStopSource::LeftExecution,
                    WorthQueryComparisonNextAction::ResolveAuthority,
                    reason,
                    counters,
                );
            }
        };

        let counters = counters.execute_right().materialize_historical();
        let historical =
            WorthQueryHistoricalPathDeclaration::retained_from_read(request.declaration.right_read);
        let (right, materialization) = match historical
            .using(retained.clone())
            .run(execution, self)
            .into_result()
        {
            Ok(completion) => completion.into_parts(),
            Err(stop) => {
                return stopped(
                    WorthQueryComparisonStopSource::RightExecution,
                    WorthQueryComparisonNextAction::ResolveAuthority,
                    format!("right historical comparison query stopped: {stop:?}"),
                    counters,
                );
            }
        };
        let pair = WorthQueryComparisonBasisPairEvidence::new(
            WorthQueryComparisonBasisFamily::CurrentToHistorical,
            WorthQueryComparisonBasisEvidence::new(
                current.workspace_name(),
                current.snapshot().clone(),
                WorthQueryComparisonMaterialization::RuntimeCurrent,
                None,
            ),
            WorthQueryComparisonBasisEvidence::new(
                retained.workspace_name(),
                retained.snapshot_identity().clone(),
                WorthQueryComparisonMaterialization::RetainedHistorical(materialization),
                None,
            ),
        );
        assemble(
            request.declaration.intent,
            request.declaration.candidate_budget,
            left,
            right,
            pair,
            counters,
        )
    }
}

impl WorthQueryComparisonExecution for (&mut WorthQueryWorkspace, &mut WorthQueryWorkspace) {
    fn execute(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: WorthQueryComparisonRequest,
    ) -> WorthQueryComparisonOutcome {
        let (left_workspace, right_workspace) = self;
        let WorthQueryComparisonPairAuthority::BranchToBranch { left, right } =
            request.context.authority.clone()
        else {
            return invalid_execution_resource();
        };
        let counters = WorthQueryComparisonJourneyCounters::validate_pair();
        if !left.matches(execution, left_workspace) || !right.matches(execution, right_workspace) {
            return stale_pair(counters);
        }

        let counters = counters.execute_left();
        let left_result =
            match execute_current_read(execution, request.declaration.left_read, left_workspace) {
                Ok(result) => result,
                Err(reason) => {
                    return stopped(
                        WorthQueryComparisonStopSource::LeftExecution,
                        WorthQueryComparisonNextAction::ResolveAuthority,
                        reason,
                        counters,
                    );
                }
            };
        let counters = counters.execute_right();
        let right_result = match execute_current_read(
            execution,
            request.declaration.right_read,
            right_workspace,
        ) {
            Ok(result) => result,
            Err(reason) => {
                return stopped(
                    WorthQueryComparisonStopSource::RightExecution,
                    WorthQueryComparisonNextAction::ResolveAuthority,
                    reason,
                    counters,
                );
            }
        };
        let pair = WorthQueryComparisonBasisPairEvidence::new(
            WorthQueryComparisonBasisFamily::BranchToBranch,
            WorthQueryComparisonBasisEvidence::new(
                left.workspace_name(),
                left.snapshot().clone(),
                WorthQueryComparisonMaterialization::RuntimeCurrent,
                Some(left.admission().admission_identity().clone()),
            ),
            WorthQueryComparisonBasisEvidence::new(
                right.workspace_name(),
                right.snapshot().clone(),
                WorthQueryComparisonMaterialization::RuntimeCurrent,
                Some(right.admission().admission_identity().clone()),
            ),
        );
        assemble(
            request.declaration.intent,
            request.declaration.candidate_budget,
            left_result,
            right_result,
            pair,
            counters,
        )
    }
}

fn execute_current_read(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    read: crate::ordinary::read::WorthQueryReadDeclaration,
    workspace: &mut WorthQueryWorkspace,
) -> Result<WorthQueryReadResult, String> {
    read.using(current())
        .run(execution, workspace)
        .into_result()
        .map(|completion| completion.into_result())
        .map_err(|stop| format!("comparison query stopped: {stop:?}"))
}

fn assemble(
    intent: super::WorthQueryComparisonIntent,
    candidate_budget: usize,
    left: WorthQueryReadResult,
    right: WorthQueryReadResult,
    pair: WorthQueryComparisonBasisPairEvidence,
    counters: WorthQueryComparisonJourneyCounters,
) -> WorthQueryComparisonOutcome {
    match intent {
        super::WorthQueryComparisonIntent::Diff => complete_diff(left, right, pair, counters),
        super::WorthQueryComparisonIntent::StructuralCorrespondence => {
            structural_correspondence(left, right, candidate_budget, pair, counters)
        }
        super::WorthQueryComparisonIntent::Lineage => {
            lineage_correspondence(left, right, pair, counters)
        }
    }
}

fn complete_diff(
    left: WorthQueryReadResult,
    right: WorthQueryReadResult,
    pair: WorthQueryComparisonBasisPairEvidence,
    counters: WorthQueryComparisonJourneyCounters,
) -> WorthQueryComparisonOutcome {
    let diff = match assemble_query_shaped_row_changes(left.rows(), right.rows()) {
        Ok(diff) => diff,
        Err(failure) => {
            return stopped(
                WorthQueryComparisonStopSource::ComparisonAssembly,
                WorthQueryComparisonNextAction::ResolveAuthority,
                failure.reason(),
                counters.record_diff_breadth(
                    failure.left_row_scan_count(),
                    failure.right_row_scan_count(),
                    0,
                ),
            );
        }
    };
    let counters = counters.record_diff_breadth(
        diff.left_row_scan_count(),
        diff.right_row_scan_count(),
        diff.row_changes().len(),
    );
    let row_changes = diff.into_row_changes();
    let change = if row_changes.is_empty() {
        WorthQueryComparisonChange::Unchanged
    } else {
        WorthQueryComparisonChange::Changed
    };
    WorthQueryComparisonOutcome::Completed(WorthQueryComparisonCompletion::new(
        left,
        right,
        pair,
        row_changes,
        change,
        counters,
    ))
}

fn structural_correspondence(
    left: WorthQueryReadResult,
    right: WorthQueryReadResult,
    candidate_budget: usize,
    pair: WorthQueryComparisonBasisPairEvidence,
    counters: WorthQueryComparisonJourneyCounters,
) -> WorthQueryComparisonOutcome {
    let [subject] = left.rows() else {
        return stopped(
            WorthQueryComparisonStopSource::CorrespondenceDenied,
            WorthQueryComparisonNextAction::NarrowCandidates,
            "structural correspondence requires exactly one subject row on the left basis",
            counters,
        );
    };
    let subject_identity = subject.identity().clone();
    let mut candidates = right
        .rows()
        .iter()
        .map(|row| row.identity().evidence_identity().as_str().to_string())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();
    resolve_correspondence(
        CorrespondenceEvaluationRequest::structural_only(
            candidates,
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            candidate_budget,
            StructuralCandidateOrderingContract::StableFingerprintOrder,
        ),
        subject_identity,
        pair,
        counters.resolve_correspondence(),
    )
}

fn lineage_correspondence(
    left: WorthQueryReadResult,
    right: WorthQueryReadResult,
    pair: WorthQueryComparisonBasisPairEvidence,
    counters: WorthQueryComparisonJourneyCounters,
) -> WorthQueryComparisonOutcome {
    let ([left_row], [right_row]) = (left.rows(), right.rows()) else {
        return stopped(
            WorthQueryComparisonStopSource::CorrespondenceDenied,
            WorthQueryComparisonNextAction::NarrowCandidates,
            "authoritative lineage comparison requires exactly one row on each basis",
            counters,
        );
    };
    resolve_correspondence(
        CorrespondenceEvaluationRequest::lineage_only(
            left_row.identity().evidence_identity().as_str(),
            right_row.identity().evidence_identity().as_str(),
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            1,
        ),
        left_row.identity().clone(),
        pair,
        counters.resolve_correspondence(),
    )
}

fn resolve_correspondence(
    request: CorrespondenceEvaluationRequest,
    subject: crate::memory_workspace::WorthQueryEntityIdentity,
    pair: WorthQueryComparisonBasisPairEvidence,
    counters: WorthQueryComparisonJourneyCounters,
) -> WorthQueryComparisonOutcome {
    match resolve_correspondence_evidence(request) {
        Ok(correspondence) if correspondence.outcome().as_denied().is_none() => {
            let posture = if correspondence.outcome().as_lineage_continuity().is_some() {
                WorthQueryComparisonCorrespondencePosture::AuthoritativeContinuity
            } else {
                WorthQueryComparisonCorrespondencePosture::Advisory
            };
            WorthQueryComparisonOutcome::Correspondence(WorthQueryComparisonCorrespondence::new(
                subject,
                correspondence,
                posture,
                pair,
                counters,
            ))
        }
        Ok(correspondence) => stopped(
            WorthQueryComparisonStopSource::CorrespondenceDenied,
            WorthQueryComparisonNextAction::NarrowCandidates,
            correspondence
                .outcome()
                .as_denied()
                .map(|denial| denial.reason())
                .unwrap_or("correspondence was denied"),
            counters,
        ),
        Err(error) => stopped(
            WorthQueryComparisonStopSource::CorrespondenceDenied,
            WorthQueryComparisonNextAction::ResolveAuthority,
            format!("correspondence resolution failed: {error:?}"),
            counters,
        ),
    }
}

#[path = "execution_stops.rs"]
mod execution_stops;
use execution_stops::{invalid_execution_resource, stale_pair, stopped};
