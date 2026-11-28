//! Value decoding precedes fresh native admission; recorded data never issues authority.
use super::*;
use crate::application_entry::mutation::WorthQueryCapturedMutationInput;
use worth_foundational::facade::{
    CanonicalDigestId, ProgramSupportObligationIdentity, ValueDecodeBounds,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationInputScope,
};
type Captured<S, B> = WorthQueryCapturedMutationInput<S, B>;
type Description<S, B> = WorthQueryOperationDescription<S, Captured<S, B>>;
impl WorthQueryOriginalOperation {
    /// Readmit a self-contained native intent directly from its recorded input.
    /// The input binding remains the decoding and admission owner.
    pub fn readmit_native<S, I, P, A>(
        identity: ProgramSupportObligationIdentity,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        bounds: ValueDecodeBounds,
    ) -> Result<WorthQueryOperationDescription<S, I>, WorthQueryOriginalOperationDenial>
    where
        S: ApplicationSchema + 'static,
        I: ApplicationMutationIntent<S> + serde::Serialize + serde::de::DeserializeOwned + Clone,
        I::Binding: ApplicationMutationBinding<S, Input = I>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey: serde::de::DeserializeOwned,
    {
        let (description, source) =
            decode::<S, I::Binding, P, A, I>(identity, request, runtime, bounds, |input| {
                input.input().clone()
            })?;
        if source.is_some() {
            return Err(WorthQueryOriginalOperationDenial::OwnershipMaterialUnavailable);
        }
        Ok(description)
    }

    /// Recover an unpinned original. Source-bound originals require `readmit_from_query`.
    pub fn readmit<S, B, P, A>(
        identity: ProgramSupportObligationIdentity,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        bounds: ValueDecodeBounds,
    ) -> Result<Description<S, B>, WorthQueryOriginalOperationDenial>
    where
        S: ApplicationSchema + 'static,
        B: ApplicationMutationBinding<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, B>,
        B::Input:
            serde::Serialize + serde::de::DeserializeOwned + ApplicationMutationInputScope<S, B>,
        B::IdempotencyKey: serde::de::DeserializeOwned,
    {
        let (description, source) =
            decode::<S, B, P, A, _>(identity, request, runtime, bounds, Captured::new)?;
        if source.is_some() {
            return Err(WorthQueryOriginalOperationDenial::OwnershipMaterialUnavailable);
        }
        Ok(description)
    }
    /// Recreate the exact observed source through its installed query and original basis.
    pub fn readmit_from_query<S, B, P, A, Q>(
        identity: ProgramSupportObligationIdentity,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        bounds: ValueDecodeBounds,
    ) -> Result<Description<S, B>, WorthQueryOriginalOperationDenial>
    where
        S: ApplicationSchema + 'static,
        B: ApplicationMutationBinding<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, B>,
        B::Input:
            serde::Serialize + serde::de::DeserializeOwned + ApplicationMutationInputScope<S, B>,
        B::IdempotencyKey: serde::de::DeserializeOwned,
        Q: ApplicationQueryBinding<
            S,
            Query = super::super::descriptor::SourceQuery<S, Captured<S, B>>,
        >,
        <Q::ResultBinding as ApplicationStructuredValueBinding>::Value:
            WorthQueryApplicationProjection<S, Q::Query> + 'static,
    {
        let (mut description, source) =
            decode::<S, B, P, A, _>(identity, request, runtime, bounds, Captured::new)?;
        let source =
            source.ok_or(WorthQueryOriginalOperationDenial::OwnershipMaterialUnavailable)?;
        use crate::application_entry::mutation::request::WorthQueryMutationExpectedSource;
        use worth_query_execution::publication_boundary::original_operation::WorthQueryReadmittedOriginalSource;
        let recovered = request
            .application
            .with_application_advancement(request.scope, |phase| {
                request.application.readmit_original_operation_source::<Q>(
                    &phase,
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    request.principal,
                    request.scope,
                    &source,
                )
            })
            .map_err(WorthQueryOriginalOperationDenial::Execution)??;
        description.source = Some(match recovered {
            WorthQueryReadmittedOriginalSource::Row(row) => {
                WorthQueryMutationExpectedSource::Row(row)
            }
            WorthQueryReadmittedOriginalSource::ResultSet(set) => {
                WorthQueryMutationExpectedSource::ResultSet(set)
            }
        });
        Ok(description)
    }
}
fn decode<S, B, P, A, I>(
    identity: ProgramSupportObligationIdentity,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    bounds: ValueDecodeBounds,
    build_intent: impl FnOnce(ApplicationEncodedInput<B::InputBinding>) -> I,
) -> Result<
    (
        WorthQueryOperationDescription<S, I>,
        Option<WorthQueryOriginalSourceData>,
    ),
    WorthQueryOriginalOperationDenial,
>
where
    S: ApplicationSchema + 'static,
    B: ApplicationMutationBinding<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, B>,
    I: ApplicationMutationIntent<S, Binding = B>,
    B::Input: serde::Serialize + serde::de::DeserializeOwned,
    B::IdempotencyKey: serde::de::DeserializeOwned,
{
    use WorthQueryOriginalOperationDenial as D;
    if !std::ptr::eq(request.application, runtime.runtime()) {
        return Err(D::ForeignHome);
    }
    request
        .application
        .with_application_advancement(request.scope, |phase| {
            let (owner, row) = runtime.readmit_original_operation_in_advancement(
                &phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                identity,
            )?;
            let recorded: RecordedDescription<
                B::IdempotencyKey,
                super::super::descriptor::Conditions<S, I>,
            > = rmp_serde::from_slice(&row.data.description)
                .map_err(|_| D::OwnershipMaterialUnavailable)?;
            let action = runtime
                .installed_program()
                .resolve_action::<B, A>()
                .map_err(|_| D::OwnershipMaterialUnavailable)?;
            let selected = runtime
                .runtime()
                .on_branch(request.branch)
                .select(&phase)
                .map_err(|_| D::OwnershipMaterialUnavailable)?;
            if action.operation_identifier() != row.data.action
                || action.binding() != row.data.binding
                || action.composition_instance() != recorded.composition
                || action.feature() != recorded.feature
                || selected.product().branch_identity().name().as_str() != row.data.branch
                || request.branch.occurrence_ordinal() != recorded.occurrence
            {
                return Err(D::OwnershipMaterialUnavailable);
            }
            let input = ApplicationEncodedInput::<B::InputBinding>::decode_retained(
                &recorded.input,
                CanonicalDigestId::new(recorded.input_identity),
                bounds,
            )
            .map_err(|_| D::OwnershipMaterialUnavailable)?;
            let route = std::num::NonZeroUsize::new(recorded.route_rounds)
                .ok_or(D::OwnershipMaterialUnavailable)?;
            let attempt = std::num::NonZeroUsize::new(recorded.attempt_rounds)
                .ok_or(D::OwnershipMaterialUnavailable)?;
            Ok((
                WorthQueryOperationDescription {
                    original: Some(OriginalCustody {
                        owner,
                        position: row.position,
                        readmission: row.position != QueryOriginalOperationPosition::Initial,
                        action: action.clone(),
                    }),
                    intent: build_intent(input.clone()),
                    encoded_input: Ok(input),
                    key: recorded.key,
                    preconditions: recorded.preconditions,
                    source: None,
                    branch: request.branch,

                    producer_round_limits:
                        crate::application_entry::ApplicationProducerRoundLimits::new(
                            route, attempt,
                        ),
                },
                recorded.source,
            ))
        })
        .map_err(D::Execution)?
}
