use crate::{
    WorthServerOperationReadinessDenial, WorthServerOperationReadinessDenialCode,
    WorthServerOperationRequestDenial, WorthServerOperationRequestDenialCode,
    WorthServerQueryHandoffDenial, WorthServerQueryHandoffDenialCode,
    WorthServerQueryHandoffDenialFacts,
};

pub(crate) fn map_operation_request_denial(
    denial: WorthServerOperationRequestDenial,
) -> WorthServerQueryHandoffDenial {
    let code = match denial.code() {
        WorthServerOperationRequestDenialCode::InvalidIdempotencyKey => {
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid
        }
        WorthServerOperationRequestDenialCode::UnknownOperationName => {
            WorthServerQueryHandoffDenialCode::UnknownOperationName
        }
        WorthServerOperationRequestDenialCode::InvalidBasisDigest => {
            WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed
        }
        WorthServerOperationRequestDenialCode::CompatibilityBindingInvalid
        | WorthServerOperationRequestDenialCode::InvalidOperationName
        | WorthServerOperationRequestDenialCode::MissingOperationName => {
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid
        }
        _ => WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
    };
    let rejected_operation_name = match denial.code() {
        WorthServerOperationRequestDenialCode::UnknownOperationName => {
            denial.detail().split('`').nth(1).map(str::to_string)
        }
        _ => None,
    };
    let denial =
        WorthServerQueryHandoffDenial::new(code, denial.diagnostics_profile(), denial.detail());
    match rejected_operation_name {
        Some(operation_name) => denial.with_facts(
            WorthServerQueryHandoffDenialFacts::default()
                .with_rejected_operation_name(operation_name),
        ),
        None => denial,
    }
}

pub(crate) fn map_readiness_denial(
    prepared_request: &crate::WorthServerCompatibilityPreparedRequest,
    denial: WorthServerOperationReadinessDenial,
) -> WorthServerQueryHandoffDenial {
    let code = match denial.code() {
        WorthServerOperationReadinessDenialCode::InvalidPreconditionInput => {
            WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid
        }
        WorthServerOperationReadinessDenialCode::PreconditionFailed => {
            WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed
        }
        WorthServerOperationReadinessDenialCode::DownstreamDeliveryRequiresReadIntent => {
            WorthServerQueryHandoffDenialCode::DownstreamDeliveryRequiresReadIntent
        }
        WorthServerOperationReadinessDenialCode::RuntimeBackedResumeUnsupported => {
            WorthServerQueryHandoffDenialCode::RuntimeBackedResumeUnsupported
        }
        WorthServerOperationReadinessDenialCode::DurableResumeDeferred => {
            WorthServerQueryHandoffDenialCode::DurableResumeDeferred
        }
        WorthServerOperationReadinessDenialCode::MissingQuerySupport
        | WorthServerOperationReadinessDenialCode::UnsupportedQuerySupport
        | WorthServerOperationReadinessDenialCode::UnsupportedProductSupport
        | WorthServerOperationReadinessDenialCode::UnknownProductSupport
        | WorthServerOperationReadinessDenialCode::FixtureOnlyProductSupport
        | WorthServerOperationReadinessDenialCode::IncompatibleSupportBasis => {
            WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
        }
    };
    let mut mapped = WorthServerQueryHandoffDenial::new(
        code,
        prepared_request
            .admission()
            .request_context()
            .diagnostics_profile(),
        denial.detail(),
    );
    if let Some(facts) = denial.facts() {
        let mut query_facts = WorthServerQueryHandoffDenialFacts::default();
        if let (Some(expected), Some(observed)) =
            (facts.expected_basis_digest(), facts.observed_basis_digest())
        {
            query_facts = query_facts.with_basis_mismatch(expected, observed);
        }
        if let (Some(expected), Some(observed)) =
            (facts.expected_validator(), facts.observed_validator())
        {
            query_facts = query_facts.with_validator_mismatch(expected, observed);
        }
        mapped = mapped.with_facts(query_facts);
    }
    mapped
}
