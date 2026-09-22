use super::*;

impl<Schema, Operation, Input, Scope> PreparedWorkflowAdvance<Schema, Operation, Input, Scope> {
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn resolve_condition_replay<
        Query,
    >(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        required: &RequiredWorkflowCondition,
        source: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            Query,
            bool,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<Option<WorkflowProgressOutcome>, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let (admission, selected_occurrence, locator, replays) = match self {
            Self::Transition {
                program,
                transition_identity_locator,
                replays,
                ..
            } => (
                &program.read_set.admission,
                program
                    .read_set
                    .lease
                    .product()
                    .product_branch()
                    .occurrence(),
                transition_identity_locator,
                replays,
            ),
            Self::AwaitingAssessment(prepared) => (
                &prepared.admitted.read_set().admission,
                prepared
                    .admitted
                    .read_set()
                    .lease
                    .product()
                    .product_branch()
                    .occurrence(),
                &prepared.layout.transition.identity,
                &prepared.replays,
            ),
            Self::AwaitingCondition(prepared) => (
                &prepared.admitted.read_set().admission,
                prepared
                    .admitted
                    .read_set()
                    .lease
                    .product()
                    .product_branch()
                    .occurrence(),
                &prepared.layout.transition.identity,
                &prepared.replays,
            ),
            Self::AwaitingOperation(prepared) => (
                &prepared.admitted.read_set().admission,
                prepared
                    .admitted
                    .read_set()
                    .lease
                    .product()
                    .product_branch()
                    .occurrence(),
                &prepared.layout.transition.identity,
                &prepared.replays,
            ),
            Self::AwaitingEvidence {
                read_set,
                transition_identity_locator,
                replays,
                ..
            }
            | Self::AwaitingApproval {
                read_set,
                transition_identity_locator,
                replays,
                ..
            }
            | Self::ReplayOnly {
                read_set,
                transition_identity_locator,
                replays,
                ..
            } => (
                &read_set.admission,
                read_set.lease.product().product_branch().occurrence(),
                transition_identity_locator,
                replays,
            ),
        };
        let replays = replays.materialize();
        let Some(replay) = replays
            .iter()
            .find(|replay| replay.identity == required.transition_identity())
        else {
            return Ok(None);
        };
        let [observed] = source.observed_sources() else {
            return Ok(None);
        };
        let Some(expected_query_identity) = runtime
            .installed_schema()
            .installed_query_identity_by_name(required.query())
        else {
            return Ok(None);
        };
        if source.rows().len() != 1
            || observed.runtime_authority != runtime.runtime.authority_identity().as_u64()
            || observed.schema_binding != runtime.installed_schema().binding_identity()
            || observed.query_identifier != required.query()
            || observed.query_identity != *expected_query_identity
            || observed.source_root() != admission.scope_entity_id()
            || observed.selected_product_occurrence() != Some(selected_occurrence)
            || &observed.branch != admission.graph_work_branch()
        {
            return Ok(None);
        }
        let binding = idempotency
            .bind_workflow_transition(&replay.identity_bytes)
            .bind_workflow_condition(&observed.idempotency_identity());
        let [resolution] = runtime
            .resolve_admitted_application_idempotencies(admission, [binding])?
            .try_into()
            .expect("one condition replay binding returns one resolution");
        Ok(match resolution.into_resolution() {
            WorthQueryApplicationIdempotencyResolution::Unseen => None,
            WorthQueryApplicationIdempotencyResolution::IntentDrift => Some(
                WorkflowProgressOutcome::Application(WorthQueryApplicationCommitOutcome::Denied(
                    WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
                )),
            ),
            WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => {
                Some(runtime.primary_provider.graph.with_runtime(|relational| {
                    super::super::project(
                        relational,
                        receipt,
                        replay.identity.clone(),
                        locator.clone(),
                        replay.node_path.clone(),
                        None,
                        None,
                        None,
                        true,
                    )
                }))
            }
        })
    }
}
