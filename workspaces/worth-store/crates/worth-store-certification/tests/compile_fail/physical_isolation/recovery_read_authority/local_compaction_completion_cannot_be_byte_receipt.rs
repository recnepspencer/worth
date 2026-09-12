use worth_store::physical_runtime::stability::StablePhysicalReadReceipt;
use worth_store_physical_isolation::CompactionReadPlanCompletion;

fn promote(completion: CompactionReadPlanCompletion) -> StablePhysicalReadReceipt {
    completion
}

fn main() {}
