//! The mutation whose decision totals a set of entries. Its operation declares
//! the entry facts as its reads, so the owner of the region totals reads them
//! through the deciding handler's reader.

use std::marker::PhantomData;

use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial, PositiveLength};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_reads, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryDecisionPlan, WorthQueryInvariantEntityIdentity, WorthQueryInvariantMutationTarget,
};

use super::facts::{
    EntryFault, EntryNumber, EntryRegion, EntrySet, EntrySetKey, EntrySetMember, EntryValueBits,
    EntryWork, SetEntry,
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
worth_query_operation_reads!(TotalRegions =>[Body,EntrySet,EntrySetMember,SetEntry]);

pub(super) struct RegionTotalsDemandBinding<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for RegionTotalsDemandBinding<Schema>
{
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

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
    type ScopeBinding = PlanarMutationScope;
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

    fn scope_field() -> BodyKey {
        BodyKey::at("BodyKey")
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

    fn scope_binding(&self) -> PlanarMutationScope {
        PlanarMutationScope::new(BodyKey::at("BodyKey"), self.scope_key.clone())
    }
}

// compiled-example: begin
pub(super) type Decision<'borrow, 'reader, 'runtime, Schema> =
    DecisionReader<'borrow, 'reader, 'runtime, Schema, RegionTotalsDemandBinding<Schema>>;

/// What the handler runs over the demanded set inside its decision.
type Run<Schema> = Box<
    dyn Fn(
            &mut Decision<'_, '_, '_, Schema>,
            &WorthQueryInvariantEntityIdentity<Schema, EntrySet>,
        ) -> WorthQueryDecisionPlan<(), PlanarMutationDenial>
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
            ) -> WorthQueryDecisionPlan<(), PlanarMutationDenial>
            + Send
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
            let scope = reader.resolve_entity(BodyKey::at("BodyKey"), input.scope_key.clone())?;
            if reader.field(&scope, PositionY::at("PositionY"))?.is_none() {
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

    fn prepare_decision(
        &self,
        input: &RegionTotalsDemand,
        reader: &mut Decision<'_, '_, '_, Schema>,
    ) -> WorthQueryDecisionPlan<WorthQueryInvariantMutationTarget<Schema, Body>, PlanarMutationDenial>
    {
        let plan = if let Some(run) = &self.0 {
            let set = match reader
                .resolve_entity(EntrySetKey::at("EntrySetKey"), input.entries.clone())
            {
                Ok(set) => set,
                Err(error) => {
                    return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(error))
                }
            };
            run(reader, &set)
        } else {
            WorthQueryDecisionPlan::immediate(HandlerResult::Completed(()))
        };
        let decided = self.decide(input, reader);
        plan.and_then(move |()| decided)
    }

    fn build_candidate(
        &self,
        input: &RegionTotalsDemand,
        target: WorthQueryInvariantMutationTarget<Schema, Body>,
        writer: &mut CandidateWriter<'_, Schema, RegionTotalsDemandBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let written = writer.projected_entity(&target).and_then(|entity| {
            writer.write_field(&entity, PositionY::at("PositionY"), input.replacement_y)
        });
        match written {
            Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                changed_vertices: 1,
            }),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

// compiled-example: end

/// The entry facts, and the operation that reads them. A decision reads the
/// demanded set's membership and five facts of every entry.
pub(super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = TotalRegions::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 4_096)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::at("BodyKey"))
        .operation_read_field(operation, PositionY::at("PositionY"))
        .operation_read_entity(operation, EntrySet::reference())
        .operation_read_field(operation, EntrySetKey::at("EntrySetKey"))
        .operation_read_relation(operation, EntrySetMember::reference())
        .operation_read_entity(operation, SetEntry::reference())
        .operation_read_field(operation, EntryNumber::at("EntryNumber"))
        .operation_read_field(operation, EntryRegion::at("EntryRegion"))
        .operation_read_field(operation, EntryValueBits::at("EntryValueBits"))
        .operation_read_field(operation, EntryWork::at("EntryWork"))
        .operation_read_field(operation, EntryFault::at("EntryFault"))
        .operation_write(operation, PositionY::at("PositionY"))
        .application_mutation_binding::<RegionTotalsDemandBinding<Schema>>()
}
