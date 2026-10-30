use worth_signal::facade::ResourceRetentionCompactionBudget;

fn main() {
    let _budget = ResourceRetentionCompactionBudget {
        retained_lifecycle_history_limit: Some(1),
        retained_denied_completion_limit: Some(2),
        retained_retry_lineage_limit: Some(3),
    };
}
