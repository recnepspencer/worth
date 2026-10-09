//! The operation a producer runs to keep one scope's output: its decision
//! totals the set of entries its input names and keeps the scope's Length.
//! A test can arm its next decisions to write an entry fact their own run
//! gathered, read by the decision first or written blind.

use std::marker::PhantomData;
use std::sync::{Mutex, PoisonError};

use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial, PositiveLength};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantEntityIdentity, WorthQueryInvariantMutationTarget,
};

use super::facts::{
    EntryFault, EntryNumber, EntryRegion, EntrySet, EntrySetKey, EntrySetMember, EntrySetWeight,
    EntryValueBits, EntryWork, SetEntry,
};
use super::*;

/// The largest set a region output's decision reads, and what one entry of
/// it may cost the decision: its membership, its five facts and its entity.
/// A set of 200 is refused at seeding: the planar turn invariant's work
/// budget pays for every entity the bootstrap touches, of any kind.
pub(super) const LARGEST_SET: usize = 160;
const DECISION_FACTS_PER_ENTRY: usize = 8;
const PROJECTION_WORK_PER_ENTRY: usize = 16;
/// The scope, the set, its weight and the entry an armed decision writes.
const DECISION_FACTS_BESIDE_ENTRIES: usize = 32;
/// The facts the largest set's decision may read.
pub(super) const DECISION_FACT_BUDGET: usize =
    DECISION_FACTS_PER_ENTRY * LARGEST_SET + DECISION_FACTS_BESIDE_ENTRIES;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(super) struct RegionOutputInput {
    pub(super) scope_key: String,
    /// The name of the set of entries the decision totals.
    pub(super) entries: String,
}

worth_query_structured_value_binding!(pub(super) RegionOutputInputBinding for RegionOutputInput {
    identity: "worth.query.certification.region-output-input.v1"
});
worth_query_operation!(pub(super) TotalRegionOutput for Schema: TopologySchemaBinding, input RegionOutputInputBinding);
worth_query_operation_reads!(TotalRegionOutput => [
    Body, BodyKey, Length, EntrySet, EntrySetKey, EntrySetWeight, EntrySetMember, SetEntry,
    EntryNumber, EntryRegion, EntryValueBits, EntryWork, EntryFault,
]);
worth_query_operation_writes!(TotalRegionOutput => [Length, EntryValueBits, BodyKey, PositionX, PositionY]);
worth_query_operation_creates!(TotalRegionOutput => [Body]);
worth_query_operation_links!(TotalRegionOutput => [PlanarSuccessor]);

mod generated_ring;

pub(super) struct RegionOutputBinding<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for RegionOutputInput {
    type Binding = RegionOutputBinding<Schema>;

    fn input(&self) -> &Self {
        self
    }

    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}

/// The scope's Length, which the candidate writes back unchanged, and the
/// entry an armed decision writes with the bits it writes.
pub(super) struct RegionOutputDecision<Schema> {
    length: Option<PositiveLength>,
    own_write: Option<(WorthQueryInvariantMutationTarget<Schema, SetEntry>, u64)>,
    generated_value: Option<u64>,
}

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for RegionOutputBinding<Schema>
{
    type Input = RegionOutputInput;
    type InputBinding = RegionOutputInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = TotalRegionOutput;
    type Decision = RegionOutputDecision<Schema>;
    type Denial = PlanarMutationDenial;
    type DenialBinding = PlanarMutationDenialBinding;
    type Output = PlanarOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;

    const IDENTITY: &'static str = "worth.query.certification.region-output.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.region-output-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.region-output-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = requirements(3, 3, 0, 14, 8192);

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

pub(super) type Decision<'borrow, 'reader, 'runtime, Schema> =
    DecisionReader<'borrow, 'reader, 'runtime, Schema, RegionOutputBinding<Schema>>;

/// What the handler runs over the input's set inside its decision.
type Run<Schema> = Box<
    dyn Fn(
            &mut Decision<'_, '_, '_, Schema>,
            &WorthQueryInvariantEntityIdentity<Schema, EntrySet>,
        ) -> Option<u64>
        + Send
        + Sync,
>;

/// Whether an armed decision reads the value it writes before writing it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OwnWriteRead {
    Observed,
    /// Only the computation's gather reads the value.
    #[cfg(feature = "test-query-execution-observer")]
    Blind,
}

/// The value armed decisions write: the entry's number and the bits.
#[derive(Clone, Copy, Debug)]
pub(super) struct OwnWrite {
    pub(super) number: u64,
    pub(super) bits: u64,
    pub(super) read: OwnWriteRead,
}

/// What armed decisions write. The tests that arm it hold the checkpoint
/// recovery guard.
static OWN_WRITE: Mutex<Option<OwnWrite>> = Mutex::new(None);

/// Arms every next decision to make `write`, or disarms them.
#[cfg(feature = "test-query-execution-observer")]
pub(super) fn arm_own_write(write: Option<OwnWrite>) {
    *OWN_WRITE.lock().unwrap_or_else(PoisonError::into_inner) = write;
}

fn own_write() -> Option<OwnWrite> {
    *OWN_WRITE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Decides a region output: runs what it was installed with over the set
/// its input names, and keeps the scope's output.
pub(super) struct RegionOutputHandler<Schema: TopologySchemaBinding>(
    Option<Run<Schema>>,
    bool,
    bool,
);

impl<Schema: TopologySchemaBinding> RegionOutputHandler<Schema> {
    /// The handler of a program that runs no region output.
    pub(super) const fn idle() -> Self {
        Self(None, false, false)
    }

    #[cfg(feature = "test-query-execution-observer")]
    pub(super) fn running(
        run: impl Fn(
                &mut Decision<'_, '_, '_, Schema>,
                &WorthQueryInvariantEntityIdentity<Schema, EntrySet>,
            ) -> Option<u64>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self(Some(Box::new(run)), false, false)
    }
}

impl<Schema: TopologySchemaBinding> RegionOutputHandler<Schema> {
    #[cfg(feature = "test-query-execution-observer")]
    pub(super) fn preserving_payload(mut self) -> Self {
        self.2 = true;
        self
    }

    #[cfg(feature = "test-query-execution-observer")]
    pub(super) fn with_generated_payload(mut self) -> Self {
        self.1 = true;
        self
    }
}

impl<Schema: TopologySchemaBinding> OperationHandler<Schema, RegionOutputBinding<Schema>>
    for RegionOutputHandler<Schema>
{
    fn decide(
        &self,
        input: &RegionOutputInput,
        reader: &mut Decision<'_, '_, '_, Schema>,
    ) -> HandlerResult<RegionOutputDecision<Schema>, PlanarMutationDenial> {
        let decided = (|| {
            let scope = reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            // A preserved artifact needs no prior-output input. Reading its
            // own Length would instead build an irrelevant self-consumption chain.
            let length = if self.2 {
                None
            } else {
                let Some(length) = reader.field(&scope, Length::reference())? else {
                    return Ok(None);
                };
                Some(length)
            };
            let generated_value = if let Some(run) = &self.0 {
                let set = reader.resolve_entity(EntrySetKey::reference(), input.entries.clone())?;
                run(reader, &set)
            } else {
                None
            };
            let own_write = match own_write() {
                Some(write) => {
                    let entry = reader.resolve_entity(EntryNumber::reference(), write.number)?;
                    if write.read == OwnWriteRead::Observed {
                        reader.field(&entry, EntryValueBits::reference())?;
                    }
                    Some((reader.mutation_target(&entry)?, write.bits))
                }
                None => None,
            };
            Ok(Some(RegionOutputDecision {
                length,
                own_write,
                generated_value,
            }))
        })();
        match decided {
            Ok(Some(decision)) => HandlerResult::Completed(decision),
            Ok(None) => HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &RegionOutputInput,
        _: &RegionOutputDecision<Schema>,
    ) -> ApplicationCandidateRequirements {
        RegionOutputBinding::<Schema>::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &RegionOutputInput,
        decision: RegionOutputDecision<Schema>,
        writer: &mut CandidateWriter<'_, Schema, RegionOutputBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let written = (|| {
            let scope = writer
                .resolve_entity(BodyKey::reference(), input.scope_key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .preserve_output::<PlanarAnchorOutput<Schema>>(&scope)
                .map_err(HandlerExecutionDenial::new)?;
            if let Some(length) = decision.length {
                writer
                    .write_field(&scope, Length::reference(), length)
                    .map_err(HandlerExecutionDenial::new)?;
            }
            if let Some(value) = decision.generated_value.filter(|_| self.1) {
                generated_ring::create(
                    writer,
                    &input.scope_key,
                    value,
                    decision.length.expect("generated payload reads its Length"),
                )?;
            }
            if let Some((target, bits)) = &decision.own_write {
                let entry = writer
                    .projected_entity(target)
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .write_field(&entry, EntryValueBits::reference(), *bits)
                    .map_err(HandlerExecutionDenial::new)?;
            }
            Ok::<_, HandlerExecutionDenial>(())
        })();
        match written {
            Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                changed_vertices: 1,
            }),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}

/// The region output's operation. Its decision budgets are the largest
/// set's, entry by entry.
pub(super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = TotalRegionOutput::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, PROJECTION_WORK_PER_ENTRY * LARGEST_SET)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, Length::reference())
        .operation_read_entity(operation, EntrySet::reference())
        .operation_read_field(operation, EntrySetKey::reference())
        .operation_read_field(operation, EntrySetWeight::reference())
        .operation_read_relation(operation, EntrySetMember::reference())
        .operation_read_entity(operation, SetEntry::reference())
        .operation_read_field(operation, EntryNumber::reference())
        .operation_read_field(operation, EntryRegion::reference())
        .operation_read_field(operation, EntryValueBits::reference())
        .operation_read_field(operation, EntryWork::reference())
        .operation_read_field(operation, EntryFault::reference())
        .operation_write(operation, Length::reference())
        .operation_write(operation, EntryValueBits::reference())
        .operation_create(operation, Body::reference())
        .operation_link(operation, PlanarSuccessor::reference())
        .operation_write(operation, BodyKey::reference())
        .operation_write(operation, PositionX::reference())
        .operation_write(operation, PositionY::reference())
        .application_mutation_binding::<RegionOutputBinding<Schema>>()
}
