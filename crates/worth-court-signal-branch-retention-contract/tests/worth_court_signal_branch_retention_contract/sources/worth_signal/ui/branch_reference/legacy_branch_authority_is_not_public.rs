use worth_signal::facade::runtime::{
    BranchTargetedTransactionRequest, SignalBranchForkRequest, SignalBranchRetirementRequest,
    SignalBranchTransactionHead,
};

fn main() {
    let _ = (
        std::any::type_name::<BranchTargetedTransactionRequest>(),
        std::any::type_name::<SignalBranchForkRequest>(),
        std::any::type_name::<SignalBranchRetirementRequest>(),
        std::any::type_name::<SignalBranchTransactionHead>(),
    );
}
