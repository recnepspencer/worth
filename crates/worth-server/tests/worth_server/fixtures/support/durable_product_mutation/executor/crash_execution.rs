use worth_server::{
    WorthServerDurableProductMutationConclusion, WorthServerDurableProductMutationExecution,
};

use super::DurableMutationCrashPoint;

pub(super) fn injected_crash_execution(
    crash: DurableMutationCrashPoint,
    basis_compared: bool,
) -> WorthServerDurableProductMutationExecution {
    let conclusion = WorthServerDurableProductMutationConclusion::failed(
        format!("injected_crash_{}", crash.as_str()),
        format!("test executor injected crash at `{}`", crash.as_str()),
    );
    if basis_compared {
        WorthServerDurableProductMutationExecution::after_basis_comparison(conclusion)
    } else {
        WorthServerDurableProductMutationExecution::before_basis_comparison(conclusion)
    }
}
