//! Cold aggregation over cases exposed by ordinary compaction owners.

use super::CompactionOwnerCaseDeclaration;

pub fn compaction_owner_case_inventory() -> impl Iterator<Item = CompactionOwnerCaseDeclaration> {
    super::cutover_delta::owner_cases()
        .chain(super::publication::owner_cases())
        .chain(super::read_plan_completion::owner_cases())
        .chain(super::reclaim_queue::owner_cases())
        .chain(super::mutation_lane_receipt::owner_cases())
}
