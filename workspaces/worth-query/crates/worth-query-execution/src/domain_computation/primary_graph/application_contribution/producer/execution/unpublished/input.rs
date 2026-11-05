//! Share the same encoded input between execution and its possible continuation.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerCommitAuthority;
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationOwnedMutationIdentities,
};

pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn prepare_input<
    S,
    B,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    key: <Operation<S, B> as ApplicationMutationBinding<S>>::IdempotencyKey,
    input: <Operation<S, B> as ApplicationMutationBinding<S>>::Input,
    edition: super::super::super::InstalledProducerEdition,
    mode: WorthQueryProducerCommitAuthority,
) -> Result<
    (
        ApplicationOwnedMutationIdentities<S, Operation<S, B>>,
        PreparedProducerPublication,
    ),
    WorthQueryOutputDemandDenial,
>
where
    S: ApplicationSchema,
    B: WorthQueryApplicationProducerBinding<S>,
{
    type Encoded<S, B> =
        ApplicationEncodedInput<<Operation<S, B> as ApplicationMutationBinding<S>>::InputBinding>;
    let layout = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<
            <Operation<S, B> as ApplicationMutationBinding<S>>::Input,
        >())
        .map_err(|_| {
            denial(
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                B::IDENTITY,
            )
        })?
        .0
        .pad_to_align();
    let bytes = layout
        .size()
        .checked_add(std::mem::size_of::<Encoded<S, B>>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Option<ContinuationState>>()))
        .ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                B::IDENTITY,
            )
        })?;
    let memory = execution
        .reserve_retained_memory(bytes as u64)
        .map_err(|stop| {
            crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
                crate::domain_computation::primary_graph::output_reuse::RetainedFactStop::Execution(
                    stop.into(),
                ),
                B::IDENTITY,
            )
        })?;
    let encoded =
        Encoded::<S, B>::encode(input).map_err(|stop| identity_unavailable(B::IDENTITY,
            worth_query_declaration::facade::application_operation::ApplicationMutationIdentityDenial::Input(stop)))?;
    let identities =
        ApplicationOwnedMutationIdentities::<S, Operation<S, B>>::with_encoded_input(key, &encoded)
            .map_err(|stop| identity_unavailable(B::IDENTITY, stop))?;
    Ok((
        identities,
        PreparedProducerPublication {
            input: Box::new(encoded),
            slot: Box::new(None),
            edition,
            mode,
            memory,
        },
    ))
}
