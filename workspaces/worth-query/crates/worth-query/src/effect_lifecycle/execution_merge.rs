//! One lowered merge: bind, prepare, admit the unique values it writes, and
//! execute on one runtime borrow, so the admission reads the head the merge
//! was prepared against.

use worth_query_execution::facade::integration::{
    WorthQueryMergeUniqueValueDenial, WorthQueryMergeUniqueValueDenialKind,
    WorthQueryPrimaryGraphIntegrationHandle,
};
use worth_relational::facade::merge::PreparedMergeExecution;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::MergeExecutionOutcome;

use super::execution::lower_runtime_error;
use super::{
    EffectExecutionDeferredKind, EffectExecutionDenialKind, RelationalEffectExecutionFailure,
};

/// The application schema whose unique fields govern a merge.
pub(crate) enum MergeUniqueValueAuthority<'graph> {
    /// A runtime with no installed application schema declares no unique
    /// field.
    NoApplicationSchema,
    /// The installed primary graph's schema decides.
    PrimaryGraph(&'graph WorthQueryPrimaryGraphIntegrationHandle),
}

impl MergeUniqueValueAuthority<'_> {
    fn admit(
        self,
        runtime: &RelationalRuntime,
        prepared: &PreparedMergeExecution,
    ) -> Result<(), RelationalEffectExecutionFailure> {
        match self {
            Self::NoApplicationSchema => Ok(()),
            Self::PrimaryGraph(graph) => graph
                .admit_merge_unique_values(runtime, prepared)
                .map_err(merge_unique_value_failure),
        }
    }
}

fn merge_unique_value_failure(
    denial: WorthQueryMergeUniqueValueDenial,
) -> RelationalEffectExecutionFailure {
    RelationalEffectExecutionFailure::Denied {
        kind: merge_unique_value_denial_kind(denial.kind()),
        message: denial.field().to_string(),
    }
}

/// A merge's unique-value refusal keeps its kind: a taken value and an
/// unavailable index read as program writes read them, and the merge's own
/// refusals keep their own kinds.
pub(super) fn merge_unique_value_denial_kind(
    kind: WorthQueryMergeUniqueValueDenialKind,
) -> EffectExecutionDenialKind {
    match kind {
        WorthQueryMergeUniqueValueDenialKind::ValueTaken => {
            EffectExecutionDenialKind::UniqueValueTaken
        }
        WorthQueryMergeUniqueValueDenialKind::IndexUnavailable => {
            EffectExecutionDenialKind::UniqueIndexUnavailable
        }
        WorthQueryMergeUniqueValueDenialKind::TargetHeadUnavailable => {
            EffectExecutionDenialKind::MergeTargetHeadUnavailable
        }
        WorthQueryMergeUniqueValueDenialKind::MergedEntityNotLive => {
            EffectExecutionDenialKind::MergedEntityNotLive
        }
        WorthQueryMergeUniqueValueDenialKind::UnreadWriteShape => {
            EffectExecutionDenialKind::MergeWriteShapeUnread
        }
    }
}

pub(crate) fn execute_lowered_merge(
    runtime: &mut RelationalRuntime,
    declaration: &crate::workflow::LoweredMergeWorkflowDeclaration,
    unique_values: MergeUniqueValueAuthority<'_>,
) -> Result<MergeExecutionOutcome, RelationalEffectExecutionFailure> {
    let prepared = runtime
        .bind_merge_execution_request(declaration.merge_request().clone())
        .map_err(merge_binding_failure)
        .and_then(|bound| {
            runtime.prepare_merge_execution(bound).map_err(|error| {
                let (kind, message) =
                    lower_runtime_error(error, EffectExecutionDenialKind::MergePreparationFailed);
                RelationalEffectExecutionFailure::Denied { kind, message }
            })
        })?;
    unique_values.admit(runtime, &prepared)?;
    runtime
        .execute_prepared_merge(prepared)
        .map_err(|error| match error {
            worth_relational::facade::merge::MergeExecutionError::Commit(error) => {
                super::relational_execution_deferred::transaction_commit(error)
            }
            other => {
                let (kind, message) =
                    lower_runtime_error(other, EffectExecutionDenialKind::MergeExecutionFailed);
                RelationalEffectExecutionFailure::Denied { kind, message }
            }
        })
}

fn merge_binding_failure(
    denial: worth_relational::facade::merge::RelationalMergeRequestBindingDenial,
) -> RelationalEffectExecutionFailure {
    match denial {
        worth_relational::facade::merge::RelationalMergeRequestBindingDenial::RetentionCapacityExhausted => {
            RelationalEffectExecutionFailure::Deferred {
                kind: EffectExecutionDeferredKind::RetentionBackpressure,
                message: format!("{denial:?}"),
            }
        }
        worth_relational::facade::merge::RelationalMergeRequestBindingDenial::RetentionIdentityExhausted => {
            RelationalEffectExecutionFailure::Denied {
                kind: EffectExecutionDenialKind::TransactionRetentionIdentityExhausted,
                message: format!("{denial:?}"),
            }
        }
        worth_relational::facade::merge::RelationalMergeRequestBindingDenial::SnapshotIdentityExhausted => {
            RelationalEffectExecutionFailure::Denied {
                kind: EffectExecutionDenialKind::SnapshotIdentityExhausted,
                message: format!("{denial:?}"),
            }
        }
        _ => RelationalEffectExecutionFailure::Denied {
            kind: EffectExecutionDenialKind::MergePreparationFailed,
            message: format!("{denial:?}"),
        },
    }
}
