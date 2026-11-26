//! Until producer-rerunning merge is active, a merge refuses typed, before
//! effects, to adopt a source-named record or drop source group lineage.
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameHolder as Holder;
use worth_relational_contracts::internal_port::lineage::data::postures::LineageBasisView as _;

use crate::merge::data::BoundExecutableMergeRecordPlan::AdoptSource;
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
        let AdoptSource(adopt) = record_plan else {
            return None;
        };
        let holder = match adopt.source_record {
            Entity(id) => Holder::Entity(id),
            Relation(id) => Holder::Relation(id),
        };
        Some((adopt.source_record.clone(), basis.holder_name(&holder)?))
    });
    drop(basis);
    if let Some((record, name)) = named {
        return Err(NamedRecord(source_branch, record, name));
    }
    let base = plan.authority_binding.merge_base_commit_id;
    let Some((commit, scheme_id)) = runtime.lineage.group_lineage_after(&source_branch, base)
    else {
        return Ok(());
    };
    Err(NamedGroupLineage(source_branch, commit, scheme_id))
}
