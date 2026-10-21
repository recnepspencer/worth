//! Construct context rows and lower them through the existing real World publication batch.
use super::*;

pub(super) fn batch(
    layout: &WorthQueryWorkflowLayout,
    source: EntityId,
    actor: WorkflowAdmittedActor,
    account_kind: KindId,
    retire_source: bool,
    defect: Option<&str>,
) -> WorkerIntentBatch {
    let mut effects = context_effects(layout, source, actor, account_kind, retire_source, defect);
    corrupt(layout, &mut effects, defect);
    let mut batch = WorkerIntentBatch::new("native-output-context");
    for effect in effects {
        batch.intents.push(match effect {
            Effect::CreateEntity {
                kind, key, fields, ..
            } => MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: source.partition_id,
                kind_id: kind,
                client_key: ClientKey::raw(key),
                fields: AspectFieldPatch::from(fields),
            })),
            Effect::CreateRelation {
                kind,
                key,
                from,
                to,
            } => MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: source.partition_id,
                kind_id: kind,
                client_key: ClientKey::raw(key),
                source: from,
                target: to,
                fields: AspectFieldPatch::default(),
            })),
            _ => unreachable!("context publisher emits creations only"),
        });
    }
    if retire_source {
        batch
            .intents
            .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id: source },
            )));
    }

    batch
}

fn context_effects(
    layout: &WorthQueryWorkflowLayout,
    source: EntityId,
    actor: WorkflowAdmittedActor,
    account_kind: KindId,
    retire_source: bool,
    defect: Option<&str>,
) -> Vec<Effect> {
    let created = CreatedEntityRef {
        partition_id: source.partition_id,
        kind_id: account_kind,
        client_key: ClientKey::raw("business-output"),
    };
    let transition = CreatedEntityRef {
        partition_id: source.partition_id,
        kind_id: layout.transition.entity_kind,
        client_key: ClientKey::raw("transition"),
    };
    let mut fields = BTreeMap::from([
        (
            layout.transition.identity.clone(),
            text_value("native-context-fixture"),
        ),
        (
            layout.transition.protocol_version.clone(),
            AspectValue::UInt64(3),
        ),
        (layout.transition.occurrence.clone(), AspectValue::UInt64(0)),
        (layout.transition.outcome.clone(), AspectValue::UInt64(1)),
        (
            layout.transition.live_membership_partition.clone(),
            AspectValue::UInt64(0),
        ),
        (
            layout.transition.live_membership_slot.clone(),
            AspectValue::UInt64(0),
        ),
        (
            layout.transition.live_membership_generation.clone(),
            AspectValue::UInt64(0),
        ),
    ]);
    // The actor is genuine; these rows deliberately claim no instance,
    // occurrence allowance or performed-command proof.
    actor.append_fields(&layout.transition.advancer, &mut fields);
    let mut effects = vec![
        Effect::CreateEntity {
            kind: account_kind,
            key: "business-output".into(),
            fields: BTreeMap::new(),
            partition: Partition::Context(source.partition_id),
        },
        Effect::CreateEntity {
            kind: transition.kind_id,
            key: "transition".into(),
            fields,
            partition: Partition::Context(source.partition_id),
        },
    ];
    attach_structural_parent(layout, source.partition_id, &mut effects);
    if defect != Some("missing") {
        append_operation_outputs(
            layout,
            transition,
            "native-context-fixture",
            [
                (
                    "source",
                    if retire_source {
                        Posture::Retire
                    } else {
                        Posture::Preserve
                    },
                    "Account",
                    EntityReference::Existing(source),
                ),
                (
                    "created",
                    Posture::Create,
                    "Account",
                    EntityReference::Created(created),
                ),
            ]
            .into_iter(),
            &mut effects,
        );
    }

    effects
}

fn corrupt(layout: &WorthQueryWorkflowLayout, effects: &mut Vec<Effect>, defect: Option<&str>) {
    for effect in &mut *effects {
        if let Effect::CreateEntity { kind, fields, .. } = effect {
            if *kind == layout.transition.entity_kind {
                if defect == Some("count") {
                    fields.insert(
                        layout.transition.output_count.clone(),
                        AspectValue::UInt64(1),
                    );
                }
                if defect == Some("overflow") {
                    fields.insert(
                        layout.transition.output_count.clone(),
                        AspectValue::UInt64(u64::MAX),
                    );
                }
                if defect == Some("downgrade") {
                    fields.insert(
                        layout.transition.protocol_version.clone(),
                        AspectValue::UInt64(2),
                    );
                }
            }
            if *kind == layout.operation_output.entity_kind {
                match defect {
                    Some("duplicate") => {
                        fields.insert(
                            layout.operation_output.role.clone(),
                            text_value("duplicated"),
                        );
                    }
                    Some("empty") => {
                        fields.insert(layout.operation_output.role.clone(), text_value(""));
                    }
                    Some("posture") => {
                        fields.insert(
                            layout.operation_output.posture.clone(),
                            AspectValue::UInt64(99),
                        );
                    }
                    _ => (),
                }
            }
        }
    }
    if defect == Some("missing-target") {
        effects.retain(|effect| !matches!(effect, Effect::CreateRelation { kind, .. } if *kind == layout.operation_output.target));
    }
}

fn attach_structural_parent(
    layout: &WorthQueryWorkflowLayout,
    partition: worth_relational::facade::identity::PartitionId,
    effects: &mut Vec<Effect>,
) {
    // Satisfy real schema ownership without manufacturing a usable workflow
    // definition or occurrence. The context reader does not interpret parents.
    let reference = |kind, key: &str| {
        EntityReference::Created(CreatedEntityRef {
            partition_id: partition,
            kind_id: kind,
            client_key: ClientKey::raw(key),
        })
    };
    for (kind, key) in [
        (layout.lineage.entity_kind, "lineage"),
        (layout.definition.entity_kind, "definition"),
        (layout.node.entity_kind, "node"),
        (layout.instance.entity_kind, "instance"),
    ] {
        effects.push(Effect::CreateEntity {
            kind,
            key: key.into(),
            fields: BTreeMap::new(),
            partition: Partition::Context(partition),
        });
    }
    let lineage = reference(layout.lineage.entity_kind, "lineage");
    let definition = reference(layout.definition.entity_kind, "definition");
    let node = reference(layout.node.entity_kind, "node");
    let instance = reference(layout.instance.entity_kind, "instance");
    for (kind, from, to) in [
        (
            layout.lineage_definition_relation,
            lineage.clone(),
            definition.clone(),
        ),
        (
            layout.definition_node_relation,
            definition.clone(),
            node.clone(),
        ),
        (
            layout.definition_start_relation,
            definition.clone(),
            node.clone(),
        ),
        (layout.instance_lineage_relation, instance.clone(), lineage),
        (
            layout.instance_definition_relation,
            instance.clone(),
            definition,
        ),
        (
            layout.instance_transition_relation,
            instance,
            reference(layout.transition.entity_kind, "transition"),
        ),
        (
            layout.transition_node_relation,
            reference(layout.transition.entity_kind, "transition"),
            node,
        ),
    ] {
        effects.push(Effect::CreateRelation {
            kind,
            key: format!("context-parent-{}", kind.0),
            from,
            to,
        });
    }
}
