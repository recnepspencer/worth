//! Immutable per-target rows published in the proposal's own native candidate.
use super::super::{captured_input::WorkflowCapturedTarget, schema::WorthQueryWorkflowLayout};
use super::WorkflowProposalMeaning;
use crate::domain_computation::{
    canonical_operation_material,
    primary_graph::application_attempt::{
        WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect,
    },
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::{
    symbols::ClientKey,
    transactions::{CreatedEntityRef, EntityReference},
};

pub(super) fn identity(proposal: &str, node: &str) -> String {
    hex(Sha256::digest(
        canonical_operation_material(vec![
            ("workflow.proposal", proposal.to_owned()),
            ("workflow.input.node", node.to_owned()),
        ])
        .as_bytes(),
    )
    .into())
}
pub(super) fn emit<E>(
    layout: &WorthQueryWorkflowLayout,
    proposal: &WorkflowProposalMeaning,
    parent: &CreatedEntityRef,
    mut emit: impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), E>,
) -> Result<(), E> {
    let l = &layout.captured_input;
    for (index, WorkflowCapturedTarget { node, contract: c }) in
        proposal.captured.iter().enumerate()
    {
        let key = format!("captured-input-{index}");
        let entity = CreatedEntityRef {
            partition_id: parent.partition_id,
            kind_id: l.entity_kind,
            client_key: ClientKey::raw(key.clone()),
        };
        let mut fields = BTreeMap::from([
            (
                l.identity.clone(),
                text(identity(proposal.identity(), node)),
            ),
            (l.node.clone(), text(node.clone())),
            (l.binding.clone(), text(c.binding.clone())),
            (l.input_type.clone(), text(c.input_type.clone())),
            (l.input.clone(), text(hex(c.input))),
            (l.key.clone(), text(hex(c.key))),
            (
                l.source_kind.clone(),
                AspectValue::UInt64(c.source.map_or(0, |s| s.0)),
            ),
        ]);
        if let Some((_, identity, partition)) = c.source {
            fields.insert(l.source.clone(), text(hex(identity)));
            fields.insert(l.source_partition.clone(), text(hex(partition)));
        }
        if let Some(identity) = c.preconditions {
            fields.insert(l.preconditions.clone(), text(hex(identity)));
        }
        emit(WorthQueryApplicationRealizedEffect::CreateEntity {
            kind: l.entity_kind,
            key,
            fields,
            partition: WorthQueryApplicationCreationPartition::Context(parent.partition_id),
        })?;
        emit(WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: l.membership,
            key: format!("proposal-input-{index}"),
            from: EntityReference::Created(parent.clone()),
            to: EntityReference::Created(entity.clone()),
        })?;
        emit(WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: l.resource,
            key: format!("input-resource-{index}"),
            from: EntityReference::Created(entity),
            to: EntityReference::Existing(c.resource),
        })?;
    }
    Ok(())
}
pub(super) fn hex(identity: [u8; 32]) -> String {
    use std::fmt::Write;
    identity
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            write!(&mut text, "{byte:02x}").expect("String write");
            text
        })
}
fn text(value: String) -> AspectValue {
    AspectValue::String(InternedString::Raw(value))
}
