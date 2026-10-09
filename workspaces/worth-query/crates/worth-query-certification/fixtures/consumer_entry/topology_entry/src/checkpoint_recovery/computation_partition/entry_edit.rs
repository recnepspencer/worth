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
    count: usize,
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
            count: 1,
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

    pub(super) fn batch(self, count: usize) -> Self {
        assert!((1..=super::region_output::LARGEST_SET).contains(&count));
        Self { count, ..self }
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
    EntryRegion, EntryValueBits, EntryWork, EntryFault,
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
        Vec<(
            WorthQueryInvariantMutationTarget<Schema, SetEntry>,
            Vec<WorthQueryInvariantMutationTarget<Schema, EntrySet>>,
        )>,
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
    // The edited Native facts are observed by decide; no unrelated output
    // query is an authoritative source for an entry edit.
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.entry-edit.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.entry-edit-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.entry-edit-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(160, 160, 320, 320, 800, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(1024 * 160 + 320 * 256),
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
        input: &EntryEdit,
        target: &EditTarget<Schema>,
    ) -> ApplicationCandidateRequirements {
        let unlinks = match target {
            EditTarget::Delete(entries) => entries.iter().map(|(_, sets)| sets.len()).sum(),
            _ => 2,
        };
        if input.count == 1 && unlinks <= 2 && input.sets.len() <= 2 {
            return ApplicationCandidateRequirements::fixed_shape(
                ApplicationCandidateCardinalityCeiling::fixed(1, 1, 2, 2, 5, 0),
                ApplicationCandidateResourceCeiling::representation_bytes(1024),
            );
        }
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(
                input.count,
                input.count,
                input.count * input.sets.len().max(2),
                unlinks,
                input.count * 5,
                0,
            ),
            ApplicationCandidateResourceCeiling::representation_bytes(
                1024 * input.count + (unlinks + input.count * input.sets.len()) * 256,
            ),
        )
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

mod declaration;
pub(super) use declaration::declare;
