use super::*;

/// The successor's own start facts, its link to the source and to every
/// effect it carries, and the source's end. Live membership moves from the
/// source to the successor, so lineage capacity is unchanged, and the
/// successor carries every step its sources took and their earliest
/// deadline, so neither bound starts afresh.
pub(super) fn successor_effects(
    layout: &WorthQueryWorkflowLayout,
    resumed: &CompiledWorkflowDefinition,
    instance_identity: &str,
    started_program_revision: &AspectValue,
    branch_occurrence: u64,
    subject: EntityId,
    (source, live_membership, carry): (EntityId, RelationId, WorkflowLineageCarry),
    performed: &[WorkflowPerformedEffect],
    advancer: crate::domain_computation::primary_graph::workflow::actor::record::WorkflowAdmittedActor,
) -> Result<Vec<WorthQueryApplicationRealizedEffect>, WorthQueryApplicationAttemptDenial> {
    let mut effects = Vec::new();
    let successor = visit_instance_start_facts(
        layout,
        resumed,
        instance_identity,
        started_program_revision,
        branch_occurrence,
        subject,
        carry,
        advancer,
        |effect| {
            effects.push(effect);
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        },
    )?;
    effects.push(WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: layout.instance_migrated_from_relation,
        key: "instance-migrated-from".to_owned(),
        from: EntityReference::Created(successor.clone()),
        to: EntityReference::Existing(source),
    });
    for (ordinal, effect) in performed.iter().enumerate() {
        effects.push(WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: layout.instance_prior_effect_relation,
            key: format!("instance-prior-effect-{ordinal}"),
            from: EntityReference::Created(successor.clone()),
            to: EntityReference::Existing(effect.transition),
        });
    }
    let mut end_fields = std::collections::BTreeMap::from([(
        layout.instance.state.clone(),
        AspectValue::UInt64(WorkflowInstanceState::Migrated.persisted_tag()),
    )]);
    advancer.append_fields(&layout.instance.end_advancer, &mut end_fields);
    effects.push(WorthQueryApplicationRealizedEffect::UpdateEntity {
        entity: "workflow-instance".to_owned(),
        entity_id: source,
        fields: end_fields,
    });
    effects.push(WorthQueryApplicationRealizedEffect::DeleteRelation {
        relation_id: live_membership,
    });
    Ok(effects)
}
