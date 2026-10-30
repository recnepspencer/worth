use worth_runtime_world::facade::{
    ExactComponentBasisKey, ExactComponentPinRequest, ObservationRetentionObligation,
    PublicationRetentionObligation, RetainedPartialRetentionObligation,
    RetentionTransferReceipt, RuntimeWorldRetentionOwner,
};

fn main() {
    let _ = (
        ExactComponentBasisKey,
        ExactComponentPinRequest,
        ObservationRetentionObligation,
        PublicationRetentionObligation,
        RetainedPartialRetentionObligation,
        RetentionTransferReceipt,
        RuntimeWorldRetentionOwner,
    );
}
