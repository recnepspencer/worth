use worth_signal::facade::ResourceOverlappingGenerationAdmission;

fn main() {
    let _ = std::mem::size_of::<ResourceOverlappingGenerationAdmission>();
    let _ = ResourceOverlappingGenerationAdmission {
        previous: panic!("private"),
        replacing: panic!("private"),
        policy_decision_digest: panic!("private"),
        old_host_work_cancellation_advisory: panic!("private"),
    };
}
