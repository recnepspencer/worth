use std::cmp::Ordering;

use worth_foundational::facade::AspectKey;
use worth_query_installation::facade::WorthQueryPortableOperationGraphReadScope as Scope;

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::decode_budget::RecordDecodeAttempt;

pub(super) fn write(output: &mut dyn BinaryEncodingSink, scope: &Scope) -> Result<(), Denial> {
    match scope {
        Scope::DerivedSource { schema, relation } => {
            output.u16(5)?;
            output.u16(1)?;
            output.text(schema)?;
            output.text(relation)
        }
        Scope::Components { schema, definition } => {
            output.u16(4)?;
            output.text(schema)?;
            super::super::operation_component_read::write(output, definition)
        }
        Scope::Entity { schema, entity } => {
            output.u16(1)?;
            output.text(schema)?;
            output.text(entity)
        }
        Scope::NativeProjection {
            schema,
            entity,
            aspect,
            contract,
            mask,
        } => {
            output.u16(2)?;
            output.text(schema)?;
            output.text(entity)?;
            output.text(aspect.as_str())?;
            super::super::foundational_aspect::write_aspect_contract(output, contract)?;
            super::super::foundational_aspect::write_projection_mask(output, mask)
        }
        Scope::Relation {
            schema,
            relation,
            from,
            to,
        } => {
            output.u16(3)?;
            output.text(schema)?;
            output.text(relation)?;
            output.text(from)?;
            output.text(to)
        }
    }
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Scope, Denial> {
    Ok(match input.u16()? {
        5 => {
            if input.u16()? != 1 {
                return Err(Denial::new(Kind::UnsupportedRecordVariant));
            }
            Scope::DerivedSource {
                schema: input.text()?.to_owned(),
                relation: input.text()?.to_owned(),
            }
        }
        4 => Scope::Components {
            schema: input.text()?.to_owned(),
            definition: super::super::operation_component_read::decode(input, budget)?,
        },
        1 => Scope::Entity {
            schema: input.text()?.to_owned(),
            entity: input.text()?.to_owned(),
        },
        2 => Scope::NativeProjection {
            schema: input.text()?.to_owned(),
            entity: input.text()?.to_owned(),
            aspect: AspectKey::new(input.text()?.to_owned())
                .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?,
            contract: super::super::foundational_aspect::decode_aspect_contract(input, budget)?,
            mask: super::super::foundational_aspect::decode_projection_mask(input, budget)?,
        },
        3 => Scope::Relation {
            schema: input.text()?.to_owned(),
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}

pub(super) fn canonical_order(left: &Scope, right: &Scope) -> Ordering {
    rank(left)
        .cmp(&rank(right))
        .then_with(|| match (left, right) {
            (
                Scope::DerivedSource {
                    schema: ls,
                    relation: lr,
                },
                Scope::DerivedSource {
                    schema: rs,
                    relation: rr,
                },
            ) => (ls, lr).cmp(&(rs, rr)),
            (
                Scope::Components {
                    schema: left_schema,
                    definition: left,
                },
                Scope::Components {
                    schema: right_schema,
                    definition: right,
                },
            ) => (left_schema, left).cmp(&(right_schema, right)),
            (
                Scope::Entity {
                    schema: left_schema,
                    entity: left_entity,
                },
                Scope::Entity {
                    schema: right_schema,
                    entity: right_entity,
                },
            ) => (left_schema, left_entity).cmp(&(right_schema, right_entity)),
            (
                Scope::NativeProjection {
                    schema: left_schema,
                    entity: left_entity,
                    aspect: left_aspect,
                    ..
                },
                Scope::NativeProjection {
                    schema: right_schema,
                    entity: right_entity,
                    aspect: right_aspect,
                    ..
                },
            ) => (left_schema, left_entity, left_aspect.as_str()).cmp(&(
                right_schema,
                right_entity,
                right_aspect.as_str(),
            )),
            (
                Scope::Relation {
                    schema: left_schema,
                    relation: left_relation,
                    from: left_from,
                    to: left_to,
                },
                Scope::Relation {
                    schema: right_schema,
                    relation: right_relation,
                    from: right_from,
                    to: right_to,
                },
            ) => (left_schema, left_relation, left_from, left_to).cmp(&(
                right_schema,
                right_relation,
                right_from,
                right_to,
            )),
            _ => Ordering::Equal,
        })
}

const fn rank(scope: &Scope) -> u8 {
    match scope {
        Scope::Components { .. } => 4,
        Scope::DerivedSource { .. } => 5,
        Scope::Entity { .. } => 1,
        Scope::NativeProjection { .. } => 2,
        Scope::Relation { .. } => 3,
    }
}
