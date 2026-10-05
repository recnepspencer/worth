//! The mutation that edits one fact of the entry sets: a set's weight, or an
//! entry's region, value or fault.

use std::marker::PhantomData;

use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};

use super::facts::{
    EntryFault, EntryNumber, EntryRegion, EntrySet, EntrySetKey, EntrySetWeight, EntryValueBits,
    SetEntry,
};
use super::*;

/// Which fact an edit changes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EntryFact {
    /// The named set's weight, to the float whose bits the edit holds.
    Weight,
    Region,
    /// The entry's value, to the float whose bits the edit holds.
    Value,
    /// The entry's fault, to the code the edit holds.
    Fault,
}

impl EntryFact {
    #[cfg(feature = "test-query-execution-observer")]
    const fn code(self) -> u64 {
        match self {
            Self::Weight => 0,
            Self::Region => 1,
            Self::Value => 2,
            Self::Fault => 3,
        }
    }

    fn of_code(code: u64) -> Self {
        match code {
            0 => Self::Weight,
            1 => Self::Region,
            2 => Self::Value,
            3 => Self::Fault,
            _ => unreachable!("an edit holds one of the four fact codes"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(super) struct EntryEdit {
    pub(super) scope_key: String,
    /// The set a weight edit changes.
    pub(super) set: String,
    /// The number of the entry every other edit changes.
    pub(super) entry: u64,
    fact: u64,
    pub(super) value: u64,
}

#[cfg(feature = "test-query-execution-observer")]
impl EntryEdit {
    pub(super) fn new(set: &str, entry: u64, fact: EntryFact, value: u64) -> Self {
        Self {
            scope_key: "anchor-a".to_owned(),
            set: set.to_owned(),
            entry,
            fact: fact.code(),
            value,
        }
    }
}

worth_query_structured_value_binding!(pub(super) EntryEditInputBinding for EntryEdit {
    identity: "worth.query.certification.entry-edit-input.v1"
});
worth_query_operation!(pub(super) EditEntry for Schema: TopologySchemaBinding, input EntryEditInputBinding);
worth_query_operation_reads!(EditEntry => [
    Body, BodyKey, EntrySet, EntrySetKey, EntrySetWeight, SetEntry, EntryNumber, EntryRegion,
    EntryValueBits, EntryFault,
]);
worth_query_operation_writes!(EditEntry => [EntrySetWeight, EntryRegion, EntryValueBits, EntryFault]);

pub(super) struct EntryEditBinding<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for EntryEdit {
    type Binding = EntryEditBinding<Schema>;

    fn input(&self) -> &Self {
        self
    }

    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}

/// The entity an edit changes.
pub(super) enum EditTarget<Schema> {
    Set(WorthQueryInvariantMutationTarget<Schema, EntrySet>),
    Entry(WorthQueryInvariantMutationTarget<Schema, SetEntry>),
}

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for EntryEditBinding<Schema>
{
    type Input = EntryEdit;
    type InputBinding = EntryEditInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = EditEntry;
    type Decision = EditTarget<Schema>;
    type Denial = PlanarMutationDenial;
    type DenialBinding = PlanarMutationDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;

    const IDENTITY: &'static str = "worth.query.certification.entry-edit.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.entry-edit-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.entry-edit-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 1, 0, 0, 1, 0),
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

pub(super) struct EntryEditHandler;

impl<Schema: TopologySchemaBinding> OperationHandler<Schema, EntryEditBinding<Schema>>
    for EntryEditHandler
{
    fn decide(
        &self,
        input: &EntryEdit,
        reader: &mut DecisionReader<'_, '_, '_, Schema, EntryEditBinding<Schema>>,
    ) -> HandlerResult<EditTarget<Schema>, PlanarMutationDenial> {
        let decided = (|| {
            reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            // A write lowers to the replacement of a fact the attempt
            // observed, and one nothing read is refused at proposal binding,
            // so the edit reads the fact it writes.
            let fact = EntryFact::of_code(input.fact);
            if fact == EntryFact::Weight {
                let set = reader.resolve_entity(EntrySetKey::reference(), input.set.clone())?;
                reader.field(&set, EntrySetWeight::reference())?;
                return Ok(EditTarget::Set(reader.mutation_target(&set)?));
            }
            let entry = reader.resolve_entity(EntryNumber::reference(), input.entry)?;
            match fact {
                EntryFact::Region => reader.field(&entry, EntryRegion::reference()).map(|_| ())?,
                EntryFact::Value => reader
                    .field(&entry, EntryValueBits::reference())
                    .map(|_| ())?,
                EntryFact::Fault => reader.field(&entry, EntryFault::reference()).map(|_| ())?,
                EntryFact::Weight => {}
            }
            Ok(EditTarget::Entry(reader.mutation_target(&entry)?))
        })();
        match decided {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &EntryEdit,
        _: &EditTarget<Schema>,
    ) -> ApplicationCandidateRequirements {
        EntryEditBinding::<Schema>::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &EntryEdit,
        target: EditTarget<Schema>,
        writer: &mut CandidateWriter<'_, Schema, EntryEditBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let written = match target {
            EditTarget::Set(set) => writer
                .projected_entity(&set)
                .and_then(|set| writer.write_field(&set, EntrySetWeight::reference(), input.value)),
            EditTarget::Entry(entry) => writer.projected_entity(&entry).and_then(|entry| {
                match EntryFact::of_code(input.fact) {
                    EntryFact::Region => {
                        writer.write_field(&entry, EntryRegion::reference(), input.value)
                    }
                    EntryFact::Value => {
                        writer.write_field(&entry, EntryValueBits::reference(), input.value)
                    }
                    EntryFact::Fault => {
                        writer.write_field(&entry, EntryFault::reference(), input.value)
                    }
                    EntryFact::Weight => unreachable!("a weight edit targets its set"),
                }
            }),
        };
        match written {
            Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                changed_vertices: 1,
            }),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

/// The edit's operation: it resolves the set or the entry it changes.
pub(super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = EditEntry::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(operation, 32)
        .operation_projection_work_budget(operation, 4_096)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_entity(operation, EntrySet::reference())
        .operation_read_field(operation, EntrySetKey::reference())
        .operation_read_field(operation, EntrySetWeight::reference())
        .operation_read_entity(operation, SetEntry::reference())
        .operation_read_field(operation, EntryNumber::reference())
        .operation_read_field(operation, EntryRegion::reference())
        .operation_read_field(operation, EntryValueBits::reference())
        .operation_read_field(operation, EntryFault::reference())
        .operation_write(operation, EntrySetWeight::reference())
        .operation_write(operation, EntryRegion::reference())
        .operation_write(operation, EntryValueBits::reference())
        .operation_write(operation, EntryFault::reference())
        .application_mutation_binding::<EntryEditBinding<Schema>>()
}
