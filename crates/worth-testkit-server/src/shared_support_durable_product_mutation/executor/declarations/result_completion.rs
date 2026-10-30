use super::*;

pub fn retry_conclusion(
    attempt: &WorthServerAdmittedDurableProductMutation,
    record: &DurableRecord,
) -> WorthServerDurableProductMutationExecution {
    let conclusion = if record.request_digest == attempt.request_digest() {
        worth_server::WorthServerDurableProductMutationConclusion::PreviouslyCommitted(
            record.completion.clone(),
        )
    } else {
        worth_server::WorthServerDurableProductMutationConclusion::IdempotencyConflict {
            bound_request_digest: record.request_digest.clone(),
        }
    };
    worth_server::WorthServerDurableProductMutationExecution::before_basis_comparison(conclusion)
}

pub fn build_completion(
    attempt: &WorthServerAdmittedDurableProductMutation,
    next_version: u64,
    product_commit_namespace: &str,
) -> Result<
    WorthServerDurableProductMutationCompletion,
    worth_server::WorthServerProductResultArtifactError,
> {
    let next_basis =
        worth_server::WorthServerProductOperationBaseDigest::new(format!("basis:{next_version}"))
            .expect("test executor basis should be canonical");
    let result = crate::shared_support_durable_product_mutation::product_result::DurableMutationProductResult::from_attempt(attempt, next_basis.value());
    let success = worth_server::WorthServerProductOperationSuccess::publish_json(
        format!("{}:{next_version}", attempt.operation_name()),
        attempt.result_contract(),
        &result,
    )?;
    Ok(
        worth_server::WorthServerDurableProductMutationCompletion::new(
            attempt,
            success,
            next_basis,
            format!(
                "product-commit:{product_commit_namespace}:{}:{next_version}",
                attempt.canonical_digest()
            ),
        )
        .expect("test executor completion should match its attempt"),
    )
}
