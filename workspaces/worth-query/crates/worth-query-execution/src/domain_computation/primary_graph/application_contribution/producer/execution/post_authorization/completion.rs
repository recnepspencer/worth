//! Handler completion and dependency identity preparation retire before commit.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEffectProgram, WorthQueryBoundSourceExpectation,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities,
};
type Program<S, B> = WorthQueryApplicationEffectProgram<
    S,
    <Operation<S, B> as ApplicationMutationBinding<S>>::Operation,
    <Operation<S, B> as ApplicationMutationBinding<S>>::Input,
    MutationScope<S, B>,
>;

#[inline(never)]
pub(super) fn prepare<Schema, Binding>(
    phase: &WorthQueryAdvancementPhase<'_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    identities: &ApplicationMutationIdentities<'_, Schema, Operation<Schema, Binding>>,
    bound_source: WorthQueryBoundSourceExpectation,
    edition: super::super::super::InstalledProducerEdition,
    successor_of: Option<[u8; 32]>,
    complete: impl FnOnce() -> Result<Program<Schema, Binding>, ProducerExecutionStop>,
) -> Result<
    (
        Program<Schema, Binding>,
        WorthQueryApplicationIdempotencyBinding,
    ),
    ProducerExecutionStop,
>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, Binding>,
        >,
{
    let mut program = complete()?;
    let identity_execution = phase
        .execution_request_for(&runtime.runtime.product_runtime)
        .map_err(|cause| {
            denial(
                WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                "producer content phase",
            )
        })?;
    let (key_identity, dependency_identity) = program
        .producer_idempotency_identities::<Operation<Schema, Binding>>(
            runtime,
            &phase
                .checked_execution_request_for(&runtime.runtime.product_runtime)
                .map_err(|stop| {
                    denial(
                        WorthQueryOutputDemandDenialKind::ExecutionRequest(stop),
                        Binding::IDENTITY,
                    )
                })?,
            *identities.key_identity(),
            worth_relational::facade::mvcc::RequiredOutputProducingImplementation::new(
                edition.build_identity(),
                edition.digest(),
            ),
            successor_of,
            identity_execution.allocation_policy(),
            runtime
                .installed_producers
                .entries
                .get(Binding::IDENTITY)
                .expect("execution retains its installed producer")
                .declaration
                .installed_output_equivalence
                .is_some(),
        )
        .map_err(|identity_denial| {
            super::super::denial::identity_failed(Binding::IDENTITY, identity_denial)
        })?;
    let idempotency = bound_source
        .bind_idempotency(
            WorthQueryApplicationIdempotencyBinding::new(
                key_identity,
                *identities.input_identity(),
            )
            .bind_mutation::<Schema, Operation<Schema, Binding>>(),
        )
        .bind_producer_dependency(&dependency_identity)
        .bind_producing_implementation(
            worth_relational::facade::mvcc::RequiredOutputProducingImplementation::new(
                edition.build_identity(),
                edition.digest(),
            ),
        );
    let program = program
        .with_output_demand_observation()
        .with_producer_required_invariants(Binding::REQUIRED_INVARIANTS);
    Ok((program, idempotency))
}
