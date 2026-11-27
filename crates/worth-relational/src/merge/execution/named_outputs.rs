//! Until producer-rerunning merge is active, a merge refuses typed, before
//! effects, to adopt a source-named record, reconcile one onto another
//! target record, or drop source named or group lineage after the base.
use worth_relational_contracts::internal_port::execution::denial_cause::RelationalExecutionDenialCause;
use worth_relational_contracts::internal_port::identity::data::primitives::LineageId;
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameHolder as Holder;
use worth_relational_contracts::internal_port::lineage::data::postures::LineageBasisView as _;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::NamingBasisUnavailable;
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;

use crate::merge::data::BoundExecutableMergeRecordPlan::{AdoptSource, Reconcile};
use crate::merge::data::MergeExecutionPreparationError::{self as Refusal, *};
use crate::merge::data::{
    BoundExecutableMergePlan, MergePlanningError, NormalizedRelationalMergeRequest,
};
use crate::transactions::data::RecordRef::{Entity, Relation};

fn storage_refusal(cause: Cause) -> Refusal {
    Planning(MergePlanningError::Execution(
        RelationalExecutionDenialCause::StorageUnavailable(RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Lineage {
                first: LineageId(0),
                last: LineageId(0),
            },
            cause,
        }),
    ))
}

fn basis_refusal(denial: NamingBasisUnavailable) -> Refusal {
    let cause = match denial {
        NamingBasisUnavailable::Storage(cause) => cause,
        _ => Cause::Damaged,
    };
    storage_refusal(cause)
}

fn demand_refusal(failure: Failure) -> Refusal {
    match failure {
        Failure::Request(stop) => Planning(MergePlanningError::Execution(stop.into())),
        Failure::StorageUnavailable(unavailable) => Planning(MergePlanningError::Execution(
            RelationalExecutionDenialCause::StorageUnavailable(unavailable),
        )),
        Failure::Content(_) => storage_refusal(Cause::Damaged),
    }
}

/// Complexity: O(record plans * log names + routed source events + group log).
pub(super) fn refuse(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    request: &NormalizedRelationalMergeRequest,
    plan: &BoundExecutableMergePlan,
) -> Result<(), Refusal> {
    let source_branch = request.source_branch().clone();
    let binding = runtime.lineage.naming_basis_binding();
    let basis = binding.read_guard(source_branch.clone());
    let named = plan.record_plans.iter().find_map(|record_plan| {
        let source = match record_plan {
            AdoptSource(adopt) => &adopt.source_record,
            Reconcile(plan) if plan.source_record != plan.target_record => &plan.source_record,
            _ => return None,
        };
        let holder = match source {
            Entity(id) => Holder::Entity(*id),
            Relation(id) => Holder::Relation(*id),
        };
        Some((source.clone(), basis.holder_name(&holder)?))
    });
    let settled = basis.settle();
    drop(basis);
    if let Some((record, name)) = named {
        return Err(NamedRecord(source_branch, record, name));
    }
    settled.map_err(basis_refusal)?;
    let base = plan.authority_binding.merge_base_commit_id;
    if let Some(commit) = runtime
        .lineage
        .named_lineage_after(execution, &source_branch, base)
        .map_err(demand_refusal)?
    {
        return Err(NamedLineage(source_branch, commit));
    }
    let Some((commit, scheme_id)) = runtime
        .lineage
        .group_lineage_after(&source_branch, base)
        .map_err(storage_refusal)?
    else {
        return Ok(());
    };
    Err(NamedGroupLineage(source_branch, commit, scheme_id))
}
