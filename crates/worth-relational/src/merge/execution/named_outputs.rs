//! Until producer-rerunning merge is active, a merge refuses typed, before
//! effects, to adopt a source-named record, reconcile one onto another
//! target record, or drop source named or group lineage after the base.
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameHolder as Holder;
use worth_relational_contracts::internal_port::lineage::data::postures::LineageBasisView as _;

use crate::merge::data::BoundExecutableMergeRecordPlan::{AdoptSource, Reconcile};
use crate::merge::data::MergeExecutionPreparationError::{self as Refusal, *};
use crate::merge::data::{BoundExecutableMergePlan, NormalizedRelationalMergeRequest};
use crate::transactions::data::RecordRef::{Entity, Relation};

/// Complexity: O(record plans * log names + published group lineage).
pub(super) fn refuse(
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
    drop(basis);
    if let Some((record, name)) = named {
        return Err(NamedRecord(source_branch, record, name));
    }
    let base = plan.authority_binding.merge_base_commit_id;
    if let Some(commit) = runtime.lineage.named_lineage_after(&source_branch, base) {
        return Err(NamedLineage(source_branch, commit));
    }
    let Some((commit, scheme_id)) = runtime.lineage.group_lineage_after(&source_branch, base)
    else {
        return Ok(());
    };
    Err(NamedGroupLineage(source_branch, commit, scheme_id))
}
