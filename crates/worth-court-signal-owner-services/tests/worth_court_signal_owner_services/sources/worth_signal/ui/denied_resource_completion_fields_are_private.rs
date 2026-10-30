use worth_signal::facade::{
    AsyncDenialId, CompletionDenialClass, DeniedResourceCompletion, ResourceAttemptId,
    ResourceBranchEpoch, ResourceGeneration, ResourceRequestId, SignalBranchId,
};

fn main() {
    let _ = DeniedResourceCompletion {
        denial_id: AsyncDenialId::new(0),
        class: CompletionDenialClass::UnknownRequest,
        request_id: ResourceRequestId::new(0),
        generation: ResourceGeneration::new(0),
        branch_epoch: ResourceBranchEpoch::new(SignalBranchId(0), 0),
        attempt: ResourceAttemptId::new(0),
        payload_byte_len: 0,
    };
}
