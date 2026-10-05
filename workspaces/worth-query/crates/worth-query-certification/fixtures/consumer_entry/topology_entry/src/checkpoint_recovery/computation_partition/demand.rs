//! The mutation whose decision totals a set of entries. Its operation declares
//! the entry facts as its reads, so the owner of the region totals reads them
//! through the deciding handler's reader.

use std::marker::PhantomData;

use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial, PositiveLength};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantEntityIdentity, WorthQueryInvariantMutationTarget,
};

use super::facts::{
    self, EntryFault, EntryNumber, EntryRegion, EntrySet, EntrySetKey, EntrySetMember,
    EntryValueBits, EntryWork, SetEntry,
};
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(super) struct RegionTotalsDemand {
    pub(super) scope_key: String,
    /// The name of the set of entries the decision totals.
    pub(super) entries: String,
    pub(super) replacement_y: PositiveLength,
}

worth_query_structured_value_binding!(pub(super) RegionTotalsDemandInputBinding for RegionTotalsDemand {
    identity: "worth.query.certification.region-totals-demand-input.v1"
});
worth_query_operation!(pub(super) TotalRegions for Schema: TopologySchemaBinding, input RegionTotalsDemandInputBinding);
worth_query_operation_reads!(TotalRegions => [
    Body, BodyKey, PositionY, EntrySet, EntrySetKey, EntrySetMember, SetEntry, EntryNumber,
    EntryRegion, EntryValueBits, EntryWork, EntryFault,
]);
worth_query_operation_writes!(TotalRegions => [PositionY]);

pub(super) struct RegionTotalsDemandBinding<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for RegionTotalsDemandBinding<Schema>
{
    type Input = RegionTotalsDemand;
    type InputBinding = RegionTotalsDemandInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = TotalRegions;
    type Decision = WorthQueryInvariantMutationTarget<Schema, Body>;
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

    const IDENTITY: &'static str = "worth.query.certification.region-totals-demand.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.region-totals-demand-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.region-totals-demand-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = requirements(0, 0, 0, 1, 1024, 4096);

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

impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for RegionTotalsDemand {
    type Binding = RegionTotalsDemandBinding<Schema>;

    fn input(&self) -> &Self {
        self
    }

    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}

pub(super) type Decision<'borrow, 'reader, 'runtime, Schema> =
    DecisionReader<'borrow, 'reader, 'runtime, Schema, RegionTotalsDemandBinding<Schema>>;

/// What the handler runs over the demanded set inside its decision.
type Run<Schema> = Box<
    dyn Fn(&mut Decision<'_, '_, '_, Schema>, &WorthQueryInvariantEntityIdentity<Schema, EntrySet>)
        + Send
        + Sync,
>;

/// Decides a demand: runs what it was installed with over the demanded set,
/// then moves the scope's source so every decision is a new one.
pub(super) struct RegionTotalsHandler<Schema: TopologySchemaBinding>(Option<Run<Schema>>);

impl<Schema: TopologySchemaBinding> RegionTotalsHandler<Schema> {
    /// The topology's own handler: nothing to run.
    pub(super) const fn idle() -> Self {
        Self(None)
    }

    pub(super) fn running(
        run: impl Fn(
                &mut Decision<'_, '_, '_, Schema>,
                &WorthQueryInvariantEntityIdentity<Schema, EntrySet>,
            ) + Send
            + Sync
            + 'static,
    ) -> Self {
        Self(Some(Box::new(run)))
    }
}

impl<Schema: TopologySchemaBinding> OperationHandler<Schema, RegionTotalsDemandBinding<Schema>>
    for RegionTotalsHandler<Schema>
{
    fn decide(
        &self,
        input: &RegionTotalsDemand,
        reader: &mut Decision<'_, '_, '_, Schema>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Body>, PlanarMutationDenial> {
        let decided = (|| {
            if let Some(run) = &self.0 {
                let set = reader.resolve_entity(EntrySetKey::reference(), input.entries.clone())?;
                run(reader, &set);
            }
            let scope = reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            if reader.field(&scope, PositionY::reference())?.is_none() {
                return Ok(None);
            }
            reader.mutation_target(&scope).map(Some)
        })();
        match decided {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &RegionTotalsDemand,
        _: &WorthQueryInvariantMutationTarget<Schema, Body>,
    ) -> ApplicationCandidateRequirements {
        RegionTotalsDemandBinding::<Schema>::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &RegionTotalsDemand,
        target: WorthQueryInvariantMutationTarget<Schema, Body>,
        writer: &mut CandidateWriter<'_, Schema, RegionTotalsDemandBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let written = writer.projected_entity(&target).and_then(|entity| {
            writer.write_field(&entity, PositionY::reference(), input.replacement_y)
        });
        match written {
            Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                changed_vertices: 1,
            }),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

/// The entry facts, and the operation that reads them. A decision reads the
/// demanded set's membership and five facts of every entry.
pub(super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = TotalRegions::reference::<Schema>();
    facts::declare(schema)
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(operation, 256)
        .operation_projection_work_budget(operation, 4_096)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, PositionY::reference())
        .operation_read_entity(operation, EntrySet::reference())
        .operation_read_field(operation, EntrySetKey::reference())
        .operation_read_relation(operation, EntrySetMember::reference())
        .operation_read_entity(operation, SetEntry::reference())
        .operation_read_field(operation, EntryNumber::reference())
        .operation_read_field(operation, EntryRegion::reference())
        .operation_read_field(operation, EntryValueBits::reference())
        .operation_read_field(operation, EntryWork::reference())
        .operation_read_field(operation, EntryFault::reference())
        .operation_write(operation, PositionY::reference())
        .application_mutation_binding::<RegionTotalsDemandBinding<Schema>>()
}
