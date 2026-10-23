//! A value correction retains the exact value its inverse restores.

pub(super) mod capability;
pub(super) mod program_adoption;
pub(in crate::checkpoint_recovery) mod recorded;
use super::{
    facts::{BodyEntrySet, EntryNumber, EntrySet, EntrySetMember, EntryValueBits, SetEntry},
    *,
};
use std::marker::PhantomData;
use worth_query_consumer_values::PlanarAdjustmentResult;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
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
    /// A derived inverse admits only the source's recorded after-image.
    pub(super) expected: Option<u64>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum EntryCorrectionDenial {
    MissingValue,
    CurrentValueChanged,
    OutsideScope,
}
worth_query_structured_value_binding!(pub(super) EntryCorrectionDenialBinding for EntryCorrectionDenial {
    identity: "worth.query.certification.entry-correction-denial.v1"
});
worth_query_structured_value_binding!(pub(super) EntryCorrectionInput for EntryCorrection {
    identity: "worth.query.certification.entry-correction-input.v1"
});
worth_query_operation!(pub(super) CorrectEntryValue for Schema: TopologySchemaBinding, input EntryCorrectionInput);
worth_query_operation_reads!(CorrectEntryValue => [Body, BodyKey, BodyEntrySet, EntrySet, EntrySetMember, SetEntry, EntryNumber, EntryValueBits]);
worth_query_operation_writes!(CorrectEntryValue => [EntryValueBits]);
pub(super) struct EntryCorrectionBinding<Schema>(PhantomData<fn() -> Schema>);
impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for EntryCorrection {
    type Binding = EntryCorrectionBinding<Schema>;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}
impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for EntryCorrectionBinding<Schema>
{
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        use worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements;
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = EntryCorrection;
    type InputBinding = EntryCorrectionInput;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = CorrectEntryValue;
    type Decision = (WorthQueryInvariantMutationTarget<Schema, SetEntry>, u64);
    type Denial = EntryCorrectionDenial;
    type DenialBinding = EntryCorrectionDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.certification.entry-correction.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.entry-correction-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.entry-correction-command.v1";

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
pub(super) struct EntryCorrectionHandler;
impl<Schema: TopologySchemaBinding> OperationHandler<Schema, EntryCorrectionBinding<Schema>>
    for EntryCorrectionHandler
{
    fn decide(
        &self,
        input: &EntryCorrection,
        reader: &mut DecisionReader<'_, '_, '_, Schema, EntryCorrectionBinding<Schema>>,
    ) -> HandlerResult<
        <EntryCorrectionBinding<Schema> as ApplicationMutationBinding<Schema>>::Decision,
        EntryCorrectionDenial,
    > {
        let decision = (|| {
            let scope = reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            let entry = reader.resolve_entity(EntryNumber::reference(), input.entry)?;
            let mut belongs = false;
            for owned in reader
                .relations_from(BodyEntrySet::reference(), &scope)
                .map_err(HandlerExecutionDenial::new)?
            {
                for member in reader
                    .relations_from(EntrySetMember::reference(), &owned.into_to())
                    .map_err(HandlerExecutionDenial::new)?
                {
                    belongs |= member.into_to().entity_id() == entry.entity_id();
                }
            }
            if !belongs {
                return Ok(None);
            }
            let value = reader.field(&entry, EntryValueBits::reference())?;
            reader
                .mutation_target(&entry)
                .map(|target| Some((target, value)))
        })();
        match decision {
            Ok(Some((_, Some(value))))
                if input.expected.is_some_and(|expected| expected != value) =>
            {
                HandlerResult::DomainDenied(EntryCorrectionDenial::CurrentValueChanged)
            }
            Ok(Some((target, Some(value)))) => HandlerResult::Completed((target, value)),
            Ok(Some((_, None))) => HandlerResult::DomainDenied(EntryCorrectionDenial::MissingValue),
            Ok(None) => HandlerResult::DomainDenied(EntryCorrectionDenial::OutsideScope),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }

    fn build_candidate(
        &self,
        input: &EntryCorrection,
        decision: <EntryCorrectionBinding<Schema> as ApplicationMutationBinding<Schema>>::Decision,
        writer: &mut CandidateWriter<'_, Schema, EntryCorrectionBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, EntryCorrectionDenial> {
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
    capability::declare(schema.correction::<recorded::RecordedEntryCorrection<Schema>>())
}

impl<Schema: TopologySchemaBinding> ApplicationDeclaredOperation<Schema> for CorrectEntryValue {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        let operation = Self::reference();
        schema
            .operation_projection_work_budget(operation, 4096)
            .operation_read_entity(operation, Body::reference())
            .operation_read_field(operation, BodyKey::reference())
            .operation_read_relation(operation, BodyEntrySet::reference())
            .operation_read_entity(operation, EntrySet::reference())
            .operation_read_relation(operation, EntrySetMember::reference())
            .operation_expected_fact(operation, BodyKey::reference())
            .operation_read_entity(operation, SetEntry::reference())
            .operation_read_field(operation, EntryNumber::reference())
            .operation_read_field(operation, EntryValueBits::reference())
            .operation_write(operation, EntryValueBits::reference())
    }
}

/// Every declaring program installs the capability-admitted correction handler.
pub(super) fn configure<Schema: TopologySchemaBinding>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<EntryCorrectionBinding<Schema>, _>(EntryCorrectionHandler)
}

impl<Schema: TopologySchemaBinding> ApplicationCapabilityMutationBinding<Schema>
    for EntryCorrectionBinding<Schema>
{
    type Capability = capability::CorrectEntryCapability<Schema>;
}
impl<Schema: TopologySchemaBinding> ApplicationMutationAdmissionBinding<Schema>
    for EntryCorrectionBinding<Schema>
{
    type Admission = ApplicationCapabilityAdmission<capability::CorrectEntryCapability<Schema>>;
}
worth_query_decl::facade::worth_query_operation_expects_fact!(CorrectEntryValue => [BodyKey]);
