use worth_signal::facade::adapters::LoweredMergePlan;

fn main() {
    let _plan = LoweredMergePlan {
        source_branch_id: unsafe { std::mem::zeroed() },
    };
}
