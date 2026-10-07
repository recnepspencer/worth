use worth_query::facade::{domain, foundation, read, runtime};

use super::super::executors::materialize_owned_workflow_stage;
use super::super::{GeometryDomain, ReadFamily, WorkflowRead};
use super::LineageEvidenceScenario;
use std::sync::{Arc, OnceLock};

pub(super) struct LineageWorkflowStageExecutor {
    scenarios: Vec<LineageEvidenceScenario>,
    target_identity: Arc<OnceLock<foundation::WorthQueryEntityIdentity>>,
}

impl LineageWorkflowStageExecutor {
    pub(super) fn new(
        scenarios: Vec<LineageEvidenceScenario>,
        target_identity: Arc<OnceLock<foundation::WorthQueryEntityIdentity>>,
    ) -> Self {
        Self {
            scenarios,
            target_identity,
        }
    }
}

impl domain::WorthQueryDomainWorkflowStageExecutor<GeometryDomain, WorkflowRead, ReadFamily>
    for LineageWorkflowStageExecutor
{
    const LOWERING_FAMILY: &'static str = "read-vertex-v1";
    const DETERMINISTIC: bool = true;
    const IDEMPOTENT_STAGE_RETRY: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const REPLAY_COMPARATOR_FAMILY: Option<&'static str> = Some("installed-workflow-exact-v1");

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(lineage_read_declaration())
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::execution_resource_support()
    }

    fn apply(
        &self,
        application: domain::WorthQueryWorkflowStageApplication<'_, '_, '_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        let (input, _computed, context, workspace) = application.into_parts();
        let publishes = context.stage().identity() == "publish";
        if publishes {
            let outcomes = self
                .scenarios
                .iter()
                .enumerate()
                .map(|(index, scenario)| {
                    let command = continuity_command(
                        *scenario,
                        index,
                        self.target_identity
                            .get()
                            .expect("lineage fixture target identity is initialized"),
                    )?;
                    let effect = context
                        .execute_mutation(command, workspace)
                        .map_err(|error| {
                            failure(format!("continuity mutation failed: {error:?}"))
                        })?;
                    if *scenario == LineageEvidenceScenario::MutationWithoutLineage {
                        Ok(None)
                    } else if let Some(observation) = correspondence_observation(
                        *scenario,
                        index,
                        self.target_identity
                            .get()
                            .expect("lineage fixture target identity is initialized"),
                    ) {
                        context
                            .execute_identity_correspondence(&effect, observation)
                            .map(Some)
                            .map_err(|error| {
                                failure(format!("identity correspondence failed: {error:?}"))
                            })
                    } else {
                        context
                            .execute_identity_evolution(&effect)
                            .map(Some)
                            .map_err(|error| {
                                failure(format!("identity evolution admission failed: {error:?}"))
                            })
                    }
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect();
            let material = materialize_owned_workflow_stage(input, context, workspace)?;
            Ok(material.with_lineage_outcomes(outcomes))
        } else {
            materialize_owned_workflow_stage(input, context, workspace)
        }
    }
}

fn lineage_read_declaration() -> &'static read::WorthQueryReadDeclaration {
    static DECLARATION: OnceLock<read::WorthQueryReadDeclaration> = OnceLock::new();
    let declaration = DECLARATION.get_or_init(|| {
        read::declare(|builder| {
            builder.local_collection(
                "Vertex",
                lineage_schema(),
                |query| query.project(read::AspectFieldSelector::new("identity", "id").unwrap()),
                |shape| {
                    shape
                        .field(read::AuthoredResultShapeField::new("identity", "id", "id").unwrap())
                },
            )
        })
        .expect("lineage collection read declaration is canonical")
    });
    declaration
}

fn lineage_schema() -> read::QuerySchemaView {
    read::QuerySchemaView::new(
        "installed-lineage-operation",
        [read::SchemaFieldView::new(
            read::AspectName::new("identity").unwrap(),
            read::FieldName::new("id").unwrap(),
            read::ScalarAspectType::String,
        )],
        [],
    )
}

impl domain::WorthQueryDomainReplaySemanticComparator<GeometryDomain, WorkflowRead, ReadFamily>
    for LineageWorkflowStageExecutor
{
    fn compare_replay_semantics(
        &self,
        original: &domain::WorthQueryWorkflowTraceSemantics,
        replay: &domain::WorthQueryWorkflowTraceSemantics,
        noise: domain::WorthQueryReplayNoiseContract,
    ) -> domain::WorthQueryReplayComparison {
        domain::compare_exact_workflow_traces(original, replay, noise)
    }
}

fn continuity_command(
    scenario: LineageEvidenceScenario,
    index: usize,
    target_identity: &foundation::WorthQueryEntityIdentity,
) -> Result<runtime::WorthQueryWriteCommand, domain::WorthQueryWorkflowStageExecutorFailure> {
    if scenario == LineageEvidenceScenario::GeneratedIdentity {
        return runtime::WorthQueryAspectMutationBuilder::new()
            .aspect("identity.id", format!("generated-lineage-{index}"))
            .naming_attach_new_target(naming_attachment(index)?)
            .build_insert("Vertex")
            .map_err(|error| failure(format!("generated command failed: {error:?}")));
    }
    if scenario == LineageEvidenceScenario::RetiredIdentity {
        return runtime::WorthQueryDeleteMutationBuilder::new()
            .build_delete(target_identity.clone())
            .map_err(|error| failure(format!("retired command failed: {error:?}")));
    }
    let binding_authority =
        runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
            runtime::WorthQueryExistingTruthBindingAuthorityLabel::new(format!(
                "lineage-binding:{index}"
            ))
            .map_err(|error| failure(error.to_string()))?,
        )
        .map_err(|error| failure(error.to_string()))?;
    let binding = runtime::WorthQueryExistingTruthTargetBinding::direct_entity(
        binding_authority,
        target_identity.clone(),
    )
    .map_err(|error| failure(error.to_string()))?
    .in_target_collection("Vertex")
    .map_err(|error| failure(error.to_string()))?;
    let prior = continuity_prior(index)?;
    let builder = runtime::WorthQueryAspectMutationBuilder::new()
        .aspect("identity.id", "lineage-authoritative-target");
    let builder = match scenario {
        LineageEvidenceScenario::PreservedIdentity => builder
            .continuity_rebind_existing_target(prior.clone(), prior.clone())
            .naming_rebind_target(naming_attachment(index)?, prior.clone(), prior),
        LineageEvidenceScenario::SingularSuccessor
        | LineageEvidenceScenario::AdvisoryCorrespondence
        | LineageEvidenceScenario::AmbiguousCorrespondence
        | LineageEvidenceScenario::ContinuityBreak
        | LineageEvidenceScenario::MutationWithoutLineage => {
            let successor = continuity_successor(index, "single")?;
            builder
                .continuity_rebind_existing_target(prior.clone(), successor.clone())
                .naming_rebind_target(naming_attachment(index)?, prior, successor)
        }
        LineageEvidenceScenario::SplitSuccessors => builder.continuity_split_successors(
            prior,
            [
                continuity_successor(index, "split-a")?,
                continuity_successor(index, "split-b")?,
            ],
        ),
        LineageEvidenceScenario::MergeSuccessor => {
            builder.continuity_rebind_merge_successor(prior, continuity_successor(index, "merge")?)
        }
        LineageEvidenceScenario::GeneratedIdentity | LineageEvidenceScenario::RetiredIdentity => {
            unreachable!("lifecycle scenarios return before continuity command construction")
        }
    };
    builder
        .build_update_existing(binding)
        .map_err(|error| failure(format!("continuity command failed: {error:?}")))
}

fn correspondence_observation(
    scenario: LineageEvidenceScenario,
    index: usize,
    subject: &foundation::WorthQueryEntityIdentity,
) -> Option<domain::WorthQueryInstalledCorrespondenceObservation> {
    let candidate_projection =
        foundation::RelationalBridgeRecordIdentityParts::entity(9, index as u64 + 1, 0)
            .terminal_projection_for_reporting();
    let candidate = foundation::WorthQueryEntityIdentity::admit_authored_entity_token(
        foundation::QueryExternalIdentityToken::new(Arc::from(candidate_projection)),
    );
    match scenario {
        LineageEvidenceScenario::AdvisoryCorrespondence => Some(
            domain::WorthQueryInstalledCorrespondenceObservation::advisory_candidate_pair(
                subject.clone(),
                candidate,
            ),
        ),
        LineageEvidenceScenario::AmbiguousCorrespondence => Some(
            domain::WorthQueryInstalledCorrespondenceObservation::ambiguous_candidate_pair(
                subject.clone(),
                candidate,
            ),
        ),
        LineageEvidenceScenario::ContinuityBreak => Some(
            domain::WorthQueryInstalledCorrespondenceObservation::explicit_continuity_break(
                subject.clone(),
                candidate,
            ),
        ),
        _ => None,
    }
}

pub(super) fn naming_attachment(
    index: usize,
) -> Result<
    runtime::WorthQueryMutationAuthorityIdentity,
    domain::WorthQueryWorkflowStageExecutorFailure,
> {
    runtime::WorthQueryMutationAuthorityIdentity::naming_attachment(
        runtime::WorthQueryNamingAttachmentAuthorityLabel::new(format!("lineage-name:{index}"))
            .map_err(|error| failure(error.to_string()))?,
    )
    .map_err(|error| failure(error.to_string()))
}

fn continuity_prior(
    index: usize,
) -> Result<
    runtime::WorthQueryMutationAuthorityIdentity,
    domain::WorthQueryWorkflowStageExecutorFailure,
> {
    runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(
        runtime::WorthQueryContinuityPriorAuthorityLabel::new(format!("lineage-prior:{index}"))
            .map_err(|error| failure(error.to_string()))?,
    )
    .map_err(|error| failure(error.to_string()))
}

fn continuity_successor(
    index: usize,
    role: &str,
) -> Result<
    runtime::WorthQueryMutationAuthorityIdentity,
    domain::WorthQueryWorkflowStageExecutorFailure,
> {
    runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(
        runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(format!(
            "lineage-successor:{index}:{role}"
        ))
        .map_err(|error| failure(error.to_string()))?,
    )
    .map_err(|error| failure(error.to_string()))
}

fn failure(detail: String) -> domain::WorthQueryWorkflowStageExecutorFailure {
    domain::WorthQueryWorkflowStageExecutorFailure::new(
        domain::WorthQueryOperationFailureClass::Indeterminate,
        detail,
    )
}
