//! The mutation that edits the entry sets: one fact of them, a set's weight
//! or an entry's region, value or fault; or their entries, made, deleted or
//! trading numbers.

use std::marker::PhantomData;

use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_deletes, worth_query_operation_links,
    worth_query_operation_reads, worth_query_operation_unlinks, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};

use super::facts::{
    EntryFault, EntryNumber, EntryRegion, EntrySet, EntrySetKey, EntrySetMember, EntrySetWeight,
    EntryValueBits, EntryWork, SetEntry,
};
use super::*;

mod membership;

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
    /// A new entry of the edit's number, value, region and work, in each of
    /// the edit's sets.
    Create,
    /// The entry, and its place in every set.
    Delete,
    /// The entry's number, traded with the entry numbered `other`.
    Swap,
}

impl EntryFact {
    #[cfg(feature = "test-query-execution-observer")]
    const fn code(self) -> u64 {
        match self {
            Self::Weight => 0,
            Self::Region => 1,
            Self::Value => 2,
            Self::Fault => 3,
            Self::Create => 4,
            Self::Delete => 5,
            Self::Swap => 6,
        }
    }

    fn of_code(code: u64) -> Self {
        match code {
            0 => Self::Weight,
            1 => Self::Region,
            2 => Self::Value,
            3 => Self::Fault,
            4 => Self::Create,
            5 => Self::Delete,
            6 => Self::Swap,
            _ => unreachable!("an edit holds one of the seven fact codes"),
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
    /// A created entry's region, or the number a swap trades with.
    other: u64,
    /// A created entry's work.
    work: u64,
    /// The sets a created entry joins.
    sets: Vec<String>,
    /// The command that makes the edit, which names the entry it creates.
    command: u64,
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
            other: 0,
            work: 0,
            sets: Vec::new(),
            command: 0,
        }
    }

    /// The edit as the command numbered `command` makes it.
    pub(super) fn commanded(self, command: u64) -> Self {
        Self { command, ..self }
    }

    /// A new entry numbered `entry` in each of `sets`.
    pub(super) fn create(sets: &[&str], entry: u64, region: u32, value: u64, work: u64) -> Self {
        Self {
            other: u64::from(region),
            work,
            sets: sets.iter().map(|set| (*set).to_owned()).collect(),
            ..Self::new("", entry, EntryFact::Create, value)
        }
    }

    pub(super) fn delete(entry: u64) -> Self {
        Self::new("", entry, EntryFact::Delete, 0)
    }

    /// The entries numbered `entry` and `other` trade numbers.
    pub(super) fn swap(entry: u64, other: u64) -> Self {
        Self {
            other,
            ..Self::new("", entry, EntryFact::Swap, 0)
        }
    }
}

worth_query_structured_value_binding!(pub(super) EntryEditInputBinding for EntryEdit {
    identity: "worth.query.certification.entry-edit-input.v1"
});
worth_query_operation!(pub(super) EditEntry for Schema: TopologySchemaBinding, input EntryEditInputBinding);
worth_query_operation_reads!(EditEntry => [
    Body, BodyKey, EntrySet, EntrySetKey, EntrySetWeight, EntrySetMember, SetEntry, EntryNumber,
    EntryRegion, EntryValueBits, EntryFault,
]);
worth_query_operation_writes!(EditEntry => [
    EntrySetWeight, EntryNumber, EntryRegion, EntryValueBits, EntryWork, EntryFault,
]);
worth_query_operation_creates!(EditEntry => [SetEntry]);
worth_query_operation_deletes!(EditEntry => [SetEntry]);
worth_query_operation_links!(EditEntry => [EntrySetMember]);
worth_query_operation_unlinks!(EditEntry => [EntrySetMember]);

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

/// The entities an edit changes.
pub(super) enum EditTarget<Schema> {
    Set(WorthQueryInvariantMutationTarget<Schema, EntrySet>),
    Entry(WorthQueryInvariantMutationTarget<Schema, SetEntry>),
    /// The sets a new entry joins.
    Create(Vec<WorthQueryInvariantMutationTarget<Schema, EntrySet>>),
    /// The entry, and the sets it leaves.
    Delete(
        WorthQueryInvariantMutationTarget<Schema, SetEntry>,
        Vec<WorthQueryInvariantMutationTarget<Schema, EntrySet>>,
    ),
    /// The entries that trade numbers, each with the number it takes.
    Swap([(WorthQueryInvariantMutationTarget<Schema, SetEntry>, u64); 2]),
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
            ApplicationCandidateCardinalityCeiling::fixed(1, 1, 2, 2, 5, 0),
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
            if matches!(
                fact,
                EntryFact::Create | EntryFact::Delete | EntryFact::Swap
            ) {
                return membership::decide(input, fact, reader);
            }
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
                _ => reader.field(&entry, EntryFault::reference()).map(|_| ())?,
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
                    _ => unreachable!("an entry edit changes one of its facts"),
                }
            }),
            membership => return membership::build(input, membership, writer),
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
        .operation_read_relation(operation, EntrySetMember::reference())
        .operation_read_entity(operation, SetEntry::reference())
        .operation_read_field(operation, EntryNumber::reference())
        .operation_read_field(operation, EntryRegion::reference())
        .operation_read_field(operation, EntryValueBits::reference())
        .operation_read_field(operation, EntryFault::reference())
        .operation_write(operation, EntrySetWeight::reference())
        .operation_write(operation, EntryNumber::reference())
        .operation_write(operation, EntryRegion::reference())
        .operation_write(operation, EntryValueBits::reference())
        .operation_write(operation, EntryWork::reference())
        .operation_write(operation, EntryFault::reference())
        .operation_create(operation, SetEntry::reference())
        .operation_delete(operation, SetEntry::reference())
        .operation_link(operation, EntrySetMember::reference())
        .operation_unlink(operation, EntrySetMember::reference())
        .application_mutation_binding::<EntryEditBinding<Schema>>()
}
