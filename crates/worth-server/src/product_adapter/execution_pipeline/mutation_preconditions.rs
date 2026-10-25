use crate::{
    WorthServerProductOperationExecutionBoundary, WorthServerProductOperationSurfaceDenial,
    WorthServerProductOperationSurfaceDenialCode, WorthServerProductOperationSurfaceDenialFacts,
};

pub(in crate::product_adapter) fn validate_product_mutation_preconditions(
    request: &crate::WorthServerOperationRequest,
    admitted_session: Option<&crate::WorthServerProductSession>,
    requires_durable_idempotency: bool,
) -> Result<(), WorthServerProductOperationSurfaceDenial> {
    if request.identity().operation_family()
        != crate::WorthServerOperationFamily::ProductApplicationMutation
    {
        return Ok(());
    }
    let Some(expected_basis) = request.identity().basis_digest() else {
        return Err(precondition_denial(
            "product mutation operations require an explicit snapshot precondition basis digest",
        ));
    };
    if let Some(observed_basis) = admitted_session.and_then(|session| session.basis_digest())
        && expected_basis != observed_basis
    {
        return Err(stale_basis_denial(expected_basis, observed_basis));
    }
    if requires_durable_idempotency && request.identity().idempotency_key().is_none() {
        return Err(precondition_denial(
            "durable product mutations require an explicit idempotency key",
        ));
    }
    Ok(())
}

fn precondition_denial(detail: &str) -> WorthServerProductOperationSurfaceDenial {
    WorthServerProductOperationSurfaceDenial::new(
        WorthServerProductOperationSurfaceDenialCode::PreconditionDenied,
        detail.to_string(),
    )
    .with_facts(
        WorthServerProductOperationSurfaceDenialFacts::default().with_execution_boundary(
            WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution,
        ),
    )
}

fn stale_basis_denial(
    expected_basis: &str,
    observed_basis: &str,
) -> WorthServerProductOperationSurfaceDenial {
    precondition_denial(&format!(
        "product snapshot precondition `{expected_basis}` did not match the admitted session basis `{observed_basis}`"
    ))
    .with_facts(
        WorthServerProductOperationSurfaceDenialFacts::default()
            .with_basis_mismatch(crate::WorthServerProductStaleBasisDenial::new(
                expected_basis,
                observed_basis,
            ))
            .with_execution_boundary(
                WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution,
            ),
    )
}
