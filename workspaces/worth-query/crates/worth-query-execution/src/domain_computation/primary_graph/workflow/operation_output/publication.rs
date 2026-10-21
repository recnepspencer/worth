//! One visitor serves candidate references and validated committed correspondence.
use crate::domain_computation::primary_graph::{
    application_attempt::{
        WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect as Effect,
    },
    workflow::schema::WorthQueryWorkflowLayout,
};
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as Posture;
use worth_relational::facade::{
    symbols::ClientKey,
    transactions::{CreatedEntityRef, EntityReference},
};

/// O(k) in the actual output inventory, never in unrelated instances or business records.
pub(in crate::domain_computation::primary_graph) fn append_operation_outputs<'output>(
    layout: &WorthQueryWorkflowLayout,
    transition: CreatedEntityRef,
    binding: &str,
    outputs: impl ExactSizeIterator<Item = (&'output str, Posture, &'output str, EntityReference)>,
    effects: &mut Vec<Effect>,
) {
    let output = &layout.operation_output;
    let fields = effects
        .iter_mut()
        .find_map(|effect| match effect {
            Effect::CreateEntity {
                kind, key, fields, ..
            } if *kind == transition.kind_id
                && Some(key.as_str()) == transition.client_key.as_raw_str() =>
            {
                Some(fields)
            }
            _ => None,
        })
        .expect("native operation transition is in its settlement effects");
    fields.insert(layout.transition.output_binding.clone(), text(binding));
    fields.insert(
        layout.transition.output_count.clone(),
        AspectValue::UInt64(outputs.len() as u64),
    );
    for (index, (role, posture, entity, target)) in outputs.enumerate() {
        let key = format!("workflow-output-{index}");
        let entry = EntityReference::Created(CreatedEntityRef {
            partition_id: transition.partition_id,
            kind_id: output.entity_kind,
            client_key: ClientKey::raw(key.clone()),
        });
        effects.push(Effect::CreateEntity {
            kind: output.entity_kind,
            key: key.clone(),
            partition: WorthQueryApplicationCreationPartition::Context(transition.partition_id),
            fields: BTreeMap::from([
                (output.role.clone(), text(role)),
                (output.entity.clone(), text(entity)),
                (
                    output.posture.clone(),
                    AspectValue::UInt64(match posture {
                        Posture::Preserve => 1,
                        Posture::Create => 2,
                        Posture::Retire => 3,
                    }),
                ),
            ]),
        });
        effects.push(Effect::CreateRelation {
            kind: output.membership,
            key: format!("{key}-membership"),
            from: EntityReference::Created(transition.clone()),
            to: entry.clone(),
        });
        effects.push(Effect::CreateRelation {
            kind: output.target,
            key: format!("{key}-target"),
            from: entry,
            to: target,
        });
    }
}

fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.to_owned()))
}
