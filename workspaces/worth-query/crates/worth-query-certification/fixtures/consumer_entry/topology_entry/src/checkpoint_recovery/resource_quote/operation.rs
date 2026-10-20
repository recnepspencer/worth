//! Genuine counted input backing and one-field output; source correction is separate.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::{
    application_operation::*, worth_query_mutation_binding, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};

pub(super) static OPERATIONS: AtomicUsize = AtomicUsize::new(0);
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct Input {
    pub scope_key: String,
    pub rows: Vec<u64>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct Edit {
    pub scope_key: String,
    pub count: u64,
}
worth_query_structured_value_binding!(pub InputValue for Input {identity:"quote-count.operation-input"});
worth_query_structured_value_binding!(pub EditValue for Edit {identity:"quote-count.edit-input"});
worth_query_operation!(pub Produce for Schema: TopologySchemaBinding, input InputValue);
worth_query_operation_reads!(Produce=>[Body,BodyKey,RequestedRows,CountedTotal]);
worth_query_operation_writes!(Produce=>[CountedTotal]);
worth_query_operation!(pub EditCount for Schema: TopologySchemaBinding, input EditValue);
worth_query_operation_reads!(EditCount=>[Body,BodyKey,RequestedRows]);
worth_query_operation_writes!(EditCount=>[RequestedRows]);

worth_query_mutation_binding! {
    pub ProduceBinding for Input, schema generic S:TopologySchemaBinding,
    identity "quote-count.produce-binding", input InputValue,operation Produce,
    result PlanarMutationResultBinding,idempotency u64,identity "quote-count.produce-command",
    decision WorthQueryInvariantMutationTarget<S,Body>,denial PlanarMutationDenialBinding,
    handler identity "quote-count.handler",outputs PlanarOutputs,
    source ApplicationQueryMutationSource<Query>,
    principal binding ConsumerPrincipalBinding::reference(),binding_type ConsumerPrincipalBinding,
        mapping ExternalPrincipalMapping,principal_entity Principal,principal_identity u64,
        identity_binding U64ApplicationValueBinding,
    scope Body,PlanarPosition,BodyKey,String,ReadOnly,NoApplicationUnit,
    field BodyKey::reference(),value_field scope_key,
    candidates creates 0,deletes 0,links 0,unlinks 0,writes 1,emits 0,
    resources retained_representation_bytes 1024
}
worth_query_mutation_binding! {
    pub EditBinding for Edit,schema generic S:TopologySchemaBinding,
    identity "quote-count.edit-binding",input EditValue,operation EditCount,
    result PlanarMutationResultBinding,idempotency u64,identity "quote-count.edit-command",
    decision WorthQueryInvariantMutationTarget<S,Body>,denial PlanarMutationDenialBinding,
    handler identity "quote-count.edit-handler",outputs NoApplicationMutationOutputs,
    source NoApplicationMutationSource,
    principal binding ConsumerPrincipalBinding::reference(),binding_type ConsumerPrincipalBinding,
        mapping ExternalPrincipalMapping,principal_entity Principal,principal_identity u64,
        identity_binding U64ApplicationValueBinding,
    scope Body,PlanarPosition,BodyKey,String,ReadOnly,NoApplicationUnit,
    field BodyKey::reference(),value_field scope_key,
    candidates creates 0,deletes 0,links 0,unlinks 0,writes 1,emits 0,
    resources retained_representation_bytes 1024
}
pub(super) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let p = Produce::reference::<S>();
    let e = EditCount::reference::<S>();
    schema
        .operation(p.definition().no_external_effect().no_aftermath().finish())
        .operation_read_entity(p, Body::reference())
        .operation_read_field(p, BodyKey::reference())
        .operation_read_field(p, RequestedRows::reference())
        .operation_read_field(p, CountedTotal::reference())
        .operation_write(p, CountedTotal::reference())
        .application_mutation_binding::<ProduceBinding<S>>()
        .operation(e.definition().no_external_effect().no_aftermath().finish())
        .operation_read_entity(e, Body::reference())
        .operation_read_field(e, BodyKey::reference())
        .operation_read_field(e, RequestedRows::reference())
        .operation_write(e, RequestedRows::reference())
        .application_mutation_binding::<EditBinding<S>>()
}
pub struct Handler;
impl<S: TopologySchemaBinding> OperationHandler<S, ProduceBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, S, ProduceBinding<S>>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<S, Body>, PlanarMutationDenial> {
        OPERATIONS.fetch_add(1, Ordering::SeqCst);
        let result = (|| {
            let entity = reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            reader.field(&entity, RequestedRows::reference())?;
            reader.field(&entity, CountedTotal::reference())?;
            reader.mutation_target(&entity)
        })();
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }
    fn candidate_requirements(
        &self,
        _: &Input,
        _: &WorthQueryInvariantMutationTarget<S, Body>,
    ) -> ApplicationCandidateRequirements {
        super::super::super::requirements(0, 0, 0, 1, 1024)
    }
    fn build_candidate(
        &self,
        input: &Input,
        target: WorthQueryInvariantMutationTarget<S, Body>,
        writer: &mut CandidateWriter<'_, S, ProduceBinding<S>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let result = (|| {
            let entity = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            let total = input.rows.iter().sum::<u64>();
            writer
                .write_field(&entity, CountedTotal::reference(), total)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .preserve_output::<PlanarAnchorOutput<S>>(&entity)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(PlanarAdjustmentResult {
                changed_vertices: 1,
            })
        })();
        match result {
            Ok(v) => HandlerResult::Completed(v),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }
}
impl<S: TopologySchemaBinding> OperationHandler<S, EditBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Edit,
        reader: &mut DecisionReader<'_, '_, '_, S, EditBinding<S>>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<S, Body>, PlanarMutationDenial> {
        let result = (|| {
            let entity = reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            reader.field(&entity, RequestedRows::reference())?;
            reader.mutation_target(&entity)
        })();
        match result {
            Ok(v) => HandlerResult::Completed(v),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }
    fn candidate_requirements(
        &self,
        _: &Edit,
        _: &WorthQueryInvariantMutationTarget<S, Body>,
    ) -> ApplicationCandidateRequirements {
        super::super::super::requirements(0, 0, 0, 1, 1024)
    }
    fn build_candidate(
        &self,
        input: &Edit,
        target: WorthQueryInvariantMutationTarget<S, Body>,
        writer: &mut CandidateWriter<'_, S, EditBinding<S>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let result = (|| {
            let entity = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&entity, RequestedRows::reference(), input.count)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(PlanarAdjustmentResult {
                changed_vertices: 1,
            })
        })();
        match result {
            Ok(v) => HandlerResult::Completed(v),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }
}
