//! A value correction retains the exact value its inverse restores.
mod capability;
use super::{
    facts::{EntryNumber, EntryValueBits, SetEntry},
    *,
};
pub(super) use capability::{seed as seed_correction_grant, CorrectEntryCapability};
use std::marker::PhantomData;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::{
    application_aftermath::*, application_operation::*, application_schema::*,
    worth_query_operation, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};

#[derive(Clone, Debug, serde::Serialize)]
pub(super) struct EntryCorrection {
    pub(super) scope_key: String,
    pub(super) entry: u64,
    pub(super) mask: u64,
}
worth_query_structured_value_binding!(pub(super) EntryCorrectionInput for EntryCorrection {
    identity: "worth.query.certification.entry-correction-input.v1"
});
worth_query_operation!(pub(super) CorrectEntryValue for Schema: TopologySchemaBinding, input EntryCorrectionInput);
worth_query_operation_reads!(CorrectEntryValue => [Body, BodyKey, SetEntry, EntryNumber, EntryValueBits]);
worth_query_operation_writes!(CorrectEntryValue => [EntryValueBits]);
pub(super) struct EntryCorrectionBinding<Schema, const WORKFLOW: bool = false>(
    PhantomData<fn() -> Schema>,
);
impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for EntryCorrection {
    type Binding = EntryCorrectionBinding<Schema>;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}
impl<Schema: TopologySchemaBinding, const WORKFLOW: bool> ApplicationMutationBinding<Schema>
    for EntryCorrectionBinding<Schema, WORKFLOW>
{
    type Input = EntryCorrection;
    type InputBinding = EntryCorrectionInput;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = CorrectEntryValue;
    type Decision = (WorthQueryInvariantMutationTarget<Schema, SetEntry>, u64);
    type Denial = PlanarMutationDenial;
    type DenialBinding = PlanarMutationDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = if WORKFLOW {
        "worth.query.certification.guarded-entry-correction.v1"
    } else {
        "worth.query.certification.entry-correction.v1"
    };
    const HANDLER_IDENTITY: &'static str = if WORKFLOW {
        "worth.query.certification.guarded-entry-correction-handler.v1"
    } else {
        "worth.query.certification.entry-correction-handler.v1"
    };
    const IDEMPOTENCY_IDENTITY: &'static str = if WORKFLOW {
        "worth.query.certification.guarded-entry-correction-command.v1"
    } else {
        "worth.query.certification.entry-correction-command.v1"
    };
    const REQUIRES_WORKFLOW_AUTHORITY: bool = WORKFLOW;
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 1, 0),
            ApplicationCandidateResourceCeiling::bounded(1024, 4096),
        );
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Body,
        PlanarPosition,
        BodyKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        BodyKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}
pub(super) struct EntryCorrectionHandler<const WORKFLOW: bool = false>;
impl<Schema: TopologySchemaBinding, const WORKFLOW: bool>
    OperationHandler<Schema, EntryCorrectionBinding<Schema, WORKFLOW>>
    for EntryCorrectionHandler<WORKFLOW>
{
    fn decide(
        &self,
        input: &EntryCorrection,
        reader: &mut DecisionReader<'_, '_, '_, Schema, EntryCorrectionBinding<Schema, WORKFLOW>>,
    ) -> HandlerResult<
        <EntryCorrectionBinding<Schema, WORKFLOW> as ApplicationMutationBinding<Schema>>::Decision,
        PlanarMutationDenial,
    > {
        let decision = (|| {
            reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            let entry = reader.resolve_entity(EntryNumber::reference(), input.entry)?;
            let value = reader.field(&entry, EntryValueBits::reference())?;
            reader.mutation_target(&entry).map(|target| (target, value))
        })();
        match decision {
            Ok((target, Some(value))) => HandlerResult::Completed((target, value)),
            Ok((_, None)) => HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }
    fn candidate_requirements(
        &self,
        _: &EntryCorrection,
        _: &<EntryCorrectionBinding<Schema, WORKFLOW> as ApplicationMutationBinding<Schema>>::Decision,
    ) -> ApplicationCandidateRequirements {
        EntryCorrectionBinding::<Schema, WORKFLOW>::CANDIDATES
    }
    fn build_candidate(
        &self,
        input: &EntryCorrection,
        decision: <EntryCorrectionBinding<Schema, WORKFLOW> as ApplicationMutationBinding<
            Schema,
        >>::Decision,
        writer: &mut CandidateWriter<'_, Schema, EntryCorrectionBinding<Schema, WORKFLOW>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let result = writer
            .projected_entity(&decision.0)
            .map_err(HandlerExecutionDenial::new)
            .and_then(|entity| {
                writer
                    .write_field(
                        &entity,
                        EntryValueBits::reference(),
                        decision.1 ^ input.mask,
                    )
                    .map_err(HandlerExecutionDenial::new)
            });
        match result {
            Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                changed_vertices: 1,
            }),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }
}
pub(super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = CorrectEntryValue::reference::<Schema>();
    let aftermath = DeclaredApplicationAftermathContract::runtime_alone(
        DeclaredCorrectionMechanism::RecordedInverse(
            DeclaredRecordedInverse::new(
                "restore-entry-value",
                DeclaredLoweringCorrespondenceRef::new("entry-value-recorded-inverse").unwrap(),
                DeclaredAftermathPostcondition::ExactPriorTruth,
                DeclaredPreImageDemand::new(
                    [DeclaredPreImageLocus::from_field(
                        EntryValueBits::reference(),
                    )],
                    1024,
                )
                .unwrap(),
            )
            .unwrap(),
        ),
    );
    capability::declare(
        schema
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .aftermath(aftermath)
                    .finish(),
            )
            .operation_decision_fact_budget(operation, 32)
            .operation_projection_work_budget(operation, 4096)
            .operation_read_entity(operation, Body::reference())
            .operation_read_field(operation, BodyKey::reference())
            .operation_read_entity(operation, SetEntry::reference())
            .operation_read_field(operation, EntryNumber::reference())
            .operation_read_field(operation, EntryValueBits::reference())
            .operation_write(operation, EntryValueBits::reference())
            .application_mutation_binding::<EntryCorrectionBinding<Schema>>()
            .application_mutation_binding::<EntryCorrectionBinding<Schema, true>>(),
    )
}

/// Every program declaring the correction installs both admission postures.
pub(super) fn configure<Schema: TopologySchemaBinding>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<EntryCorrectionBinding<Schema>, _>(EntryCorrectionHandler)?;
    setup.handler::<EntryCorrectionBinding<Schema, true>, _>(EntryCorrectionHandler::<true>)
}
