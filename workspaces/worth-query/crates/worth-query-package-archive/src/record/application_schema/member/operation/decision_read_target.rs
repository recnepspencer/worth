//! The decision-read target wire: own-scope and shared-owner peer Parent
//! reads, entities, fields and relations, each under its own tag.
use super::{tagged_text, write_three_texts};
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget;

pub(in crate::record::application_schema::member) fn write_decision_read_target(
    output: &mut dyn BinaryEncodingSink,
    target: &ApplicationOperationDecisionReadTarget,
) -> Result<(), Denial> {
    match target {
        ApplicationOperationDecisionReadTarget::ManagedIndex { parent } => {
            output.u16(4)?;
            super::super::super::query_binding_reference::write(output, parent)
        }
        ApplicationOperationDecisionReadTarget::SharedOwnerManagedIndex {
            parent,
            relation,
            owner,
            member,
        } => {
            output.u16(5)?;
            super::super::super::query_binding_reference::write(output, parent)?;
            write_three_texts(output, relation, owner, member)
        }
        ApplicationOperationDecisionReadTarget::Entity { entity } => tagged_text(output, 1, entity),
        ApplicationOperationDecisionReadTarget::Field {
            entity,
            aspect,
            field,
        } => {
            output.u16(2)?;
            write_three_texts(output, entity, aspect, field)
        }
        ApplicationOperationDecisionReadTarget::Relation { relation, from, to } => {
            output.u16(3)?;
            write_three_texts(output, relation, from, to)
        }
    }
}

pub(in crate::record::application_schema::member) fn decode_decision_read_target(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationOperationDecisionReadTarget, Denial> {
    Ok(match input.u16()? {
        4 => ApplicationOperationDecisionReadTarget::ManagedIndex {
            parent: super::super::super::query_binding_reference::decode(input)?,
        },
        5 => ApplicationOperationDecisionReadTarget::SharedOwnerManagedIndex {
            parent: super::super::super::query_binding_reference::decode(input)?,
            relation: input.text()?.to_owned(),
            owner: input.text()?.to_owned(),
            member: input.text()?.to_owned(),
        },
        1 => ApplicationOperationDecisionReadTarget::Entity {
            entity: input.text()?.to_owned(),
        },
        2 => ApplicationOperationDecisionReadTarget::Field {
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            field: input.text()?.to_owned(),
        },
        3 => ApplicationOperationDecisionReadTarget::Relation {
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}
