use super::*;
use worth_query_decl::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_host::facade::primary_graph::WorthQueryApplicationEntityKey;

pub(super) fn apply(
    input: &ActorLifecycleInput,
    targets: Targets,
    writer: &mut CandidateWriter<'_, DocumentRetentionSchema, ActorLifecycleBinding>,
) -> Result<(), HandlerExecutionDenial> {
    let document = writer
        .projected_entity(&targets.document)
        .map_err(HandlerExecutionDenial::new)?;
    let mapping = writer
        .projected_entity(&targets.mapping)
        .map_err(HandlerExecutionDenial::new)?;
    let grants = targets
        .grants
        .iter()
        .map(|grant| {
            writer
                .projected_entity(grant)
                .map_err(HandlerExecutionDenial::new)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(actor) = targets.actor {
        let actor = writer
            .projected_entity(&actor)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .unlink(MappingTarget::reference(), &mapping, &actor)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .unlink(WorkflowDocumentOwner::reference(), &actor, &document)
            .map_err(HandlerExecutionDenial::new)?;
        for grant in &grants {
            writer
                .unlink(WorkflowGrantGrantor::reference(), &actor, grant)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .unlink(WorkflowGrantGrantee::reference(), &actor, grant)
                .map_err(HandlerExecutionDenial::new)?;
        }
        writer
            .write_field(
                &mapping,
                MappingStatusField::reference(),
                WorthQueryPrincipalMappingStatus::Disabled,
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .delete_entity(Principal::reference(), &actor)
            .map_err(HandlerExecutionDenial::new)?;
    } else {
        let actor = writer
            .create_entity_in_context(
                &document,
                Principal::reference(),
                WorthQueryApplicationEntityKey::new("actor-generation-successor")
                    .map_err(HandlerExecutionDenial::new)?,
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&actor, PrincipalIdentityField::reference(), 1_u64)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .link(
                MappingTarget::reference(),
                "actor-generation-mapping",
                &mapping,
                &actor,
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .link(
                WorkflowDocumentOwner::reference(),
                "actor-generation-subject",
                &actor,
                &document,
            )
            .map_err(HandlerExecutionDenial::new)?;
        for (index, grant) in grants.iter().enumerate() {
            writer
                .link(
                    WorkflowGrantGrantor::reference(),
                    format!("actor-generation-grantor-{index}"),
                    &actor,
                    grant,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    WorkflowGrantGrantee::reference(),
                    format!("actor-generation-grantee-{index}"),
                    &actor,
                    grant,
                )
                .map_err(HandlerExecutionDenial::new)?;
        }
        writer
            .write_field(
                &mapping,
                MappingStatusField::reference(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .map_err(HandlerExecutionDenial::new)?;
    }
    let _ = input;
    Ok(())
}
