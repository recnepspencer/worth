use super::*;
use std::marker::PhantomData;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryCurrentOutputSelection,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(super) struct ChainInput {
    pub scope_key: String,
    /// Every upstream output this node consumes; its decision reads each
    /// one's Length. A diamond's shared dependent consumes two.
    pub upstreams: Vec<ChainUpstream>,
    pub value: PositiveLength,
}

/// One consumed output: a root planar output, or another chain node's.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(super) struct ChainUpstream {
    pub key: String,
    pub root: bool,
}

/// Each completed decision's scope and the consumed upstream Lengths it read.
static DECISIONS: std::sync::Mutex<Vec<(String, Vec<u64>)>> = std::sync::Mutex::new(Vec::new());

/// The upstream Lengths each decision for `scope_key` read since the last
/// take, in order. Other scopes' decisions are discarded.
#[cfg(feature = "test-query-execution-observer")]
pub(super) fn take_decisions(scope_key: &str) -> Vec<Vec<u64>> {
    take_all_decisions()
        .into_iter()
        .filter_map(|(scope, values)| (scope == scope_key).then_some(values))
        .collect()
}

/// Observe the existing decision ledger without consuming another oracle's evidence.
#[cfg(feature = "test-query-execution-observer")]
pub(super) fn decisions_snapshot() -> Vec<(String, Vec<u64>)> {
    DECISIONS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// Every decision since the last take, in order: its scope and the upstream
/// Lengths it read.
#[cfg(feature = "test-query-execution-observer")]
pub(super) fn take_all_decisions() -> Vec<(String, Vec<u64>)> {
    std::mem::take(
        &mut *DECISIONS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    )
}
/// A request to cancel from inside one scope's next decision, so its refresh
/// is interrupted after its row was claimed.
#[allow(clippy::type_complexity)]
static CANCEL_DURING_DECISION: std::sync::Mutex<
    Option<(String, authentication::WorthQueryCancellationSource)>,
> = std::sync::Mutex::new(None);

/// Cancels `cancellation` while `scope_key`'s next decision runs.
#[cfg(feature = "test-query-execution-observer")]
pub(super) fn cancel_during_next_decision(
    scope_key: &str,
    cancellation: authentication::WorthQueryCancellationSource,
) {
    *CANCEL_DURING_DECISION
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) =
        Some((scope_key.to_owned(), cancellation));
}

worth_query_structured_value_binding!(pub(super) ChainInputBinding for ChainInput {
    identity: "worth.query.certification.consumed-chain-input.v1"
});
worth_query_operation!(pub(super) PublishChain for Schema: TopologySchemaBinding, input ChainInputBinding);
worth_query_operation_reads!(PublishChain => [Body, BodyKey, Length, PositionY]);
worth_query_operation_writes!(PublishChain => [Length]);

pub(super) struct ChainBinding<Schema>(PhantomData<fn() -> Schema>);
impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for ChainInput {
    type Binding = ChainBinding<Schema>;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}
impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema> for ChainBinding<Schema> {
    type Input = ChainInput;
    type InputBinding = ChainInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = PublishChain;
    type Decision = ();
    type Denial = PlanarMutationDenial;
    type DenialBinding = PlanarMutationDenialBinding;
    type Output = PlanarOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PlanarOutputQuery>;
    const IDENTITY: &'static str = "worth.query.certification.consumed-chain-operation.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.consumed-chain-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.consumed-chain-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = requirements(0, 0, 0, 1, 8_192);
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

pub(super) struct ChainHandler;
impl<Schema: TopologySchemaBinding> OperationHandler<Schema, ChainBinding<Schema>>
    for ChainHandler
{
    fn decide(
        &self,
        input: &ChainInput,
        reader: &mut DecisionReader<'_, '_, '_, Schema, ChainBinding<Schema>>,
    ) -> HandlerResult<(), PlanarMutationDenial> {
        // The target's actual decision read establishes effect selection for
        // the candidate; each upstream read below establishes a consumed edge.
        let target = match reader.resolve_entity(BodyKey::reference(), input.scope_key.clone()) {
            Ok(entity) => entity,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&target, Length::reference()) {
            Ok(Some(value)) if value == input.value => {}
            Ok(_) => return HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        // A real decision fact, absent from the source projection and prepared
        // input key, can remove the previously consumed diamond edges.
        let drop_upstreams = if input.scope_key == "diamond-join" {
            match reader.field(&target, PositionY::reference()) {
                Ok(Some(value)) => value == length(105),
                Ok(None) => {
                    return HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate)
                }
                Err(error) => return HandlerResult::ExecutionDenied(error),
            }
        } else {
            false
        };
        let mut consumed = Vec::with_capacity(input.upstreams.len());
        for upstream in input.upstreams.iter().filter(|_| !drop_upstreams) {
            match consumed_length(upstream, reader) {
                HandlerResult::Completed(value) => consumed.push(PositiveLength::get(&value)),
                HandlerResult::DomainDenied(denial) => return HandlerResult::DomainDenied(denial),
                HandlerResult::ExecutionDenied(error) => {
                    return HandlerResult::ExecutionDenied(error)
                }
                HandlerResult::Cancelled => return HandlerResult::Cancelled,
                HandlerResult::DeadlineExceeded => return HandlerResult::DeadlineExceeded,
            }
        }
        let mut cancel = CANCEL_DURING_DECISION
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if cancel
            .as_ref()
            .is_some_and(|(scope, _)| *scope == input.scope_key)
        {
            cancel.take().expect("the armed scope matched").1.cancel();
        }
        drop(cancel);
        DECISIONS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push((input.scope_key.clone(), consumed));
        HandlerResult::Completed(())
    }
    fn candidate_requirements(&self, _: &ChainInput, _: &()) -> ApplicationCandidateRequirements {
        ChainBinding::<Schema>::CANDIDATES
    }
    fn build_candidate(
        &self,
        input: &ChainInput,
        _: (),
        writer: &mut CandidateWriter<'_, Schema, ChainBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let result = (|| {
            let entity = writer
                .resolve_entity(BodyKey::reference(), input.scope_key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .preserve_output::<PlanarAnchorOutput<Schema>>(&entity)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&entity, Length::reference(), input.value)
                .map_err(HandlerExecutionDenial::new)?;
            Ok::<_, HandlerExecutionDenial>(PlanarAdjustmentResult {
                changed_vertices: 1,
            })
        })();
        match result {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}

/// The Length the upstream's actual current output carries. The
/// `current_output` read is what records the consumed edge.
fn consumed_length<Schema: TopologySchemaBinding>(
    upstream: &ChainUpstream,
    reader: &mut DecisionReader<'_, '_, '_, Schema, ChainBinding<Schema>>,
) -> HandlerResult<PositiveLength, PlanarMutationDenial> {
    let entity = match reader.resolve_entity(BodyKey::reference(), upstream.key.clone()) {
        Ok(entity) => entity,
        Err(error) => return HandlerResult::ExecutionDenied(error),
    };
    let selected = if upstream.root {
        reader.current_output::<PlanarOutputFamily, Body>(&entity)
    } else {
        reader.current_output::<ChainFamily, Body>(&entity)
    };
    let output = match selected {
        Ok(WorthQueryCurrentOutputSelection::Unique(output)) => output,
        Ok(_) => return HandlerResult::DomainDenied(PlanarMutationDenial::CurrentOutputMissing),
        Err(error) => return HandlerResult::ExecutionDenied(error),
    };
    match reader.field(&output, BodyKey::reference()) {
        Ok(Some(key)) if key == upstream.key => {}
        Ok(_) => return HandlerResult::DomainDenied(PlanarMutationDenial::UnexpectedCurrentOutput),
        Err(error) => return HandlerResult::ExecutionDenied(error),
    }
    match reader.field(&output, Length::reference()) {
        Ok(Some(value)) => HandlerResult::Completed(value),
        Ok(None) => HandlerResult::DomainDenied(PlanarMutationDenial::CurrentOutputMissing),
        Err(error) => HandlerResult::ExecutionDenied(error),
    }
}

pub(super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = PublishChain::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        // The actual consumed-output lookup verifies upstream lineage/native
        // currentness inside the handler's projection allowance.
        .operation_projection_work_budget(operation, 4_096)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, Length::reference())
        .operation_read_field(operation, PositionY::reference())
        .operation_write(operation, Length::reference())
        .application_mutation_binding::<ChainBinding<Schema>>()
}
