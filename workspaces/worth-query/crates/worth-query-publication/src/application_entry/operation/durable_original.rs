//! Original description DATA joins the existing Query obligation and native World custody.
use super::*;
use worth_query_execution::publication_boundary::original_operation::{
    QueryOriginalOperationData, QueryOriginalOperationPosition, WorthQueryOriginalOperationDenial,
    WorthQueryOriginalOperationOwner, WorthQueryOriginalSourceData,
};
mod readmission;

#[derive(serde::Serialize, serde::Deserialize)]
struct RecordedDescription<Key, Conditions> {
    composition: String,
    feature: String,
    occurrence: u64,
    input: Vec<u8>,
    input_identity: [u8; 32],
    key: Key,
    preconditions: Conditions,
    route_rounds: usize,
    attempt_rounds: usize,
    source: Option<WorthQueryOriginalSourceData>,
}

/// Descriptive lookup for the same original. It contains no runtime or request authority.
#[must_use = "readmit this original under a fresh Home and authenticated request"]
pub struct WorthQueryOriginalOperation {
    owner: WorthQueryOriginalOperationOwner,
}
impl WorthQueryOriginalOperation {
    pub fn identity(&self) -> worth_foundational::facade::ProgramSupportObligationIdentity {
        self.owner.identity()
    }
    pub fn admit<S: ApplicationSchema>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryOriginalOperationDenial> {
        request.application.admit_original_operation_owner(
            &self.owner,
            &worth_query_execution::publication_boundary::program_publication_access(),
        )
    }
}

pub(super) struct OriginalCustody {
    pub owner: WorthQueryOriginalOperationOwner,
    pub position: QueryOriginalOperationPosition,
    pub readmission: bool,
    pub action: worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
}
impl OriginalCustody {
    pub fn mark<S: ApplicationSchema>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
        position: QueryOriginalOperationPosition,
    ) -> Result<(), WorthQueryOriginalOperationDenial> {
        runtime.progress_original_operation_in_advancement(
            phase,
            &worth_query_execution::publication_boundary::program_publication_access(),
            &self.owner,
            Some(position),
        )?;
        self.position = position;
        Ok(())
    }
}

impl<S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>>
    WorthQueryOperationDescription<S, I>
{
    /// Record immutable meaning before a Directory operation can retain its runtime.
    /// A publication refusal returns this same description, including its minted identity.
    pub fn retain_durable_operation<P, A>(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        <<A::Delivery as worth_query_declaration::facade::application_program::ApplicationActionDelivery<S, I::Binding>>::Mode as WorthQueryOperationModeFactory<S,I,P,A>>::Operation,
        (Self, WorthQueryOperationConstructionDenial),
    >
    where
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        <A::Delivery as worth_query_declaration::facade::application_program::ApplicationActionDelivery<S, I::Binding>>::Mode: WorthQueryOperationModeFactory<S,I,P,A>,
        MutationInput<S, I>: serde::Serialize,
        <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey: serde::Serialize,
{
        if let Err(cause) = self.retain_original_for_request::<P, A>(request, runtime) {
            return Err((self, cause));
        }
        self.retain_operation::<P, A>(runtime)
    }

    pub(super) fn retain_original_for_request<P, A>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), WorthQueryOperationConstructionDenial>
    where
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        MutationInput<S, I>: serde::Serialize,
        <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey: serde::Serialize,
    {
        if !std::ptr::eq(request.application, runtime.runtime()) || request.branch != self.branch {
            return Err(WorthQueryOperationConstructionDenial::ForeignRuntime);
        }
        runtime
            .runtime()
            .with_application_advancement(request.scope, |phase| {
                self.retain_original_in_phase::<P, A>(&phase, runtime)
            })
            .map_err(|cause| {
                WorthQueryOperationConstructionDenial::Original(
                    WorthQueryOriginalOperationDenial::Execution(cause),
                )
            })?
            .map_err(WorthQueryOperationConstructionDenial::Original)
    }

    fn retain_original_in_phase<P, A>(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), WorthQueryOriginalOperationDenial>
    where
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        MutationInput<S, I>: serde::Serialize,
        <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey: serde::Serialize,
    {
        use WorthQueryOriginalOperationDenial as D;
        let access = worth_query_execution::publication_boundary::program_publication_access();
        if !runtime
            .runtime()
            .original_operation_requires_record(&access)
        {
            return Ok(());
        }
        if let Some(original) = &self.original {
            runtime
                .runtime()
                .admit_original_operation_owner(&original.owner, &access)?;
            runtime.readmit_original_operation_in_advancement(
                phase,
                &access,
                original.owner.identity(),
            )?;
            return Ok(());
        }
        // Durable document history and branch retirement are assigned to C2.
        if self.history.is_some() {
            return Err(D::OwnershipMaterialUnavailable);
        }
        let action = runtime
            .installed_program()
            .resolve_action::<I::Binding, A>()
            .map_err(|_| D::OwnershipMaterialUnavailable)?;
        let input = self
            .encoded_input
            .as_ref()
            .map_err(|_| D::OwnershipMaterialUnavailable)?;
        let captured = input
            .capture_for_retention()
            .map_err(|_| D::OwnershipMaterialUnavailable)?;
        let selected = runtime
            .runtime()
            .on_branch(self.branch)
            .select(phase)
            .map_err(|_| D::OwnershipMaterialUnavailable)?;
        let source = self.source.as_ref().map(|source| match source {
            crate::application_entry::mutation::request::WorthQueryMutationExpectedSource::Row(row) => row.capture_original_operation_source(false, &access),
            crate::application_entry::mutation::request::WorthQueryMutationExpectedSource::ResultSet(set) => set.source().capture_original_operation_source(true, &access),
        }).transpose()?;
        let description = rmp_serde::to_vec(&RecordedDescription {
            composition: action.composition_instance().to_owned(),
            feature: action.feature().to_owned(),
            occurrence: self.branch.occurrence_ordinal(),
            input: captured.recorded_bytes().to_vec(),
            input_identity: *input.identity(),
            key: &self.key,
            preconditions: &self.preconditions,
            route_rounds: self.producer_round_limits.per_route().get(),
            attempt_rounds: self.producer_round_limits.per_attempt().get(),
            source,
        })
        .map_err(|_| D::OwnershipMaterialUnavailable)?;
        let data = QueryOriginalOperationData {
            revision: *runtime.installed_program().revision().as_bytes(),
            program: runtime.installed_program().identity().as_str().to_owned(),
            action: action.operation_identifier().to_owned(),
            binding: action.binding().to_owned(),
            branch: selected
                .product()
                .branch_identity()
                .name()
                .as_str()
                .to_owned(),
            description,
        };
        let (owner, pending) =
            runtime.retain_original_operation_in_advancement(phase, &access, data)?;
        self.original = Some(OriginalCustody {
            owner,
            position: QueryOriginalOperationPosition::Initial,
            readmission: false,
            action: action.clone(),
        });
        if let Some(cause) = pending {
            return Err(D::Obligation(cause));
        }
        Ok(())
    }
}

impl<S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>>
    WorthQueryOperationDescription<S, I>
{
    pub(super) fn admit_original_retention<P>(
        &self,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), WorthQueryOperationConstructionDenial> {
        runtime
            .runtime()
            .admit_original_operation_retention(
                self.original.as_ref().map(|original| &original.owner),
                &worth_query_execution::publication_boundary::program_publication_access(),
            )
            .map_err(WorthQueryOperationConstructionDenial::Original)
    }
    pub(super) fn admit_original_action(
        &self,
        action: &worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
    ) -> Result<(), WorthQueryOperationConstructionDenial> {
        if self
            .original
            .as_ref()
            .is_some_and(|original| original.action != *action)
        {
            return Err(WorthQueryOperationConstructionDenial::Original(
                WorthQueryOriginalOperationDenial::OwnershipMaterialUnavailable,
            ));
        }
        Ok(())
    }
    pub(super) fn original_lookup(
        &self,
    ) -> Result<WorthQueryOriginalOperation, WorthQueryOriginalOperationDenial> {
        self.original
            .as_ref()
            .map(|original| WorthQueryOriginalOperation {
                owner: original.owner,
            })
            .ok_or(WorthQueryOriginalOperationDenial::OwnershipMaterialUnavailable)
    }
    pub(super) fn admit_original_discharge(&self) -> Result<(), WorthQueryOperationCloseDenial> {
        if self
            .original
            .as_ref()
            .is_some_and(|original| original.readmission)
        {
            return Err(WorthQueryOperationCloseDenial::NativeObligationRetained);
        }
        Ok(())
    }
    pub(super) fn discharge_original(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryOperationCloseDenial> {
        self.admit_original_discharge()?;
        if let Some(original) = &self.original {
            request
                .application
                .with_application_advancement(request.scope, |phase| {
                    request
                        .application
                        .progress_original_operation_in_advancement(
                        &phase,
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        &original.owner,
                        None,
                    )
                })
                .map_err(WorthQueryOperationCloseDenial::Execution)?
                .map_err(WorthQueryOperationCloseDenial::Original)?;
            self.original = None;
        }
        Ok(())
    }
}
