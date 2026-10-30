pub use {
    worth_server::WorthServerDurableProductMutationConclusion,
    worth_server::WorthServerDurableProductMutationExecution,
};

pub use crate::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint;

pub fn injected_crash_execution(
    crash: DurableMutationCrashPoint,
    basis_compared: bool,
) -> WorthServerDurableProductMutationExecution {
    let conclusion = worth_server::WorthServerDurableProductMutationConclusion::failed(
        format!("injected_crash_{}", crash.as_str()),
        format!("test executor injected crash at `{}`", crash.as_str()),
    );
    if basis_compared {
        worth_server::WorthServerDurableProductMutationExecution::after_basis_comparison(conclusion)
    } else {
        worth_server::WorthServerDurableProductMutationExecution::before_basis_comparison(
            conclusion,
        )
    }
}
