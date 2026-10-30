use worth_store::physical_runtime::{
    PhysicalRecordOpen, PhysicalRecordResidencyPolicyBuilder,
};

fn attach_raw_policy(
    request: PhysicalRecordOpen,
    policy: PhysicalRecordResidencyPolicyBuilder,
) {
    let _ = request.with_residency_policy(policy);
}

fn main() {}
