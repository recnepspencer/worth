use std::cmp::Ordering;

use worth_query_installation::facade::WorthQueryPortableOperationTouchScope as Scope;

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::decode_budget::RecordDecodeAttempt;

pub(super) fn write(output: &mut dyn BinaryEncodingSink, scope: &Scope) -> Result<(), Denial> {
    match scope {
        Scope::InitializeRelationField {
            schema,
            relation,
            from,
            to,
            contract,
            field_path,
        } => {
            output.u16(6)?;
            for text in [schema, relation, from, to] {
                output.text(text)?;
            }
            super::super::foundational_aspect::write_aspect_contract(output, contract)?;
            super::super::foundational_aspect::write_field_path(output, field_path)
        }
        Scope::CreateEntity { schema, entity } => tagged_entity(output, 1, schema, entity),
        Scope::DeleteEntity { schema, entity } => tagged_entity(output, 2, schema, entity),
        Scope::WriteField {
            schema,
            entity,
            contract,
            field_path,
        } => {
            output.u16(3)?;
            output.text(schema)?;
            output.text(entity)?;
            super::super::foundational_aspect::write_aspect_contract(output, contract)?;
            super::super::foundational_aspect::write_field_path(output, field_path)
        }
        Scope::LinkRelation {
            schema,
            relation,
            from,
            to,
        } => {
            output.u16(4)?;
            output.text(schema)?;
            output.text(relation)?;
            output.text(from)?;
            output.text(to)
        }
        Scope::UnlinkRelation {
            schema,
            relation,
            from,
            to,
        } => {
            output.u16(5)?;
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
        1 => Scope::CreateEntity {
            schema: input.text()?.to_owned(),
            entity: input.text()?.to_owned(),
        },
        2 => Scope::DeleteEntity {
            schema: input.text()?.to_owned(),
            entity: input.text()?.to_owned(),
        },
        3 => Scope::WriteField {
            schema: input.text()?.to_owned(),
            entity: input.text()?.to_owned(),
            contract: super::super::foundational_aspect::decode_aspect_contract(input, budget)?,
            field_path: super::super::foundational_aspect::decode_field_path(input, budget)?,
        },
        4 => decode_relation(input, RelationTouch::Link)?,
        5 => decode_relation(input, RelationTouch::Unlink)?,
        6 => Scope::InitializeRelationField {
            schema: input.text()?.to_owned(),
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
            contract: super::super::foundational_aspect::decode_aspect_contract(input, budget)?,
            field_path: super::super::foundational_aspect::decode_field_path(input, budget)?,
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}

pub(super) fn canonical_order(left: &Scope, right: &Scope) -> Ordering {
    rank(left)
        .cmp(&rank(right))
        .then_with(|| same_variant_order(left, right))
}

fn same_variant_order(left: &Scope, right: &Scope) -> Ordering {
    match (left, right) {
        (
            Scope::CreateEntity { schema, entity } | Scope::DeleteEntity { schema, entity },
            Scope::CreateEntity {
                schema: other_schema,
                entity: other_entity,
            }
            | Scope::DeleteEntity {
                schema: other_schema,
                entity: other_entity,
            },
        ) => (schema, entity).cmp(&(other_schema, other_entity)),
        (Scope::WriteField { .. }, Scope::WriteField { .. }) => write_field_order(left, right),
        (
            Scope::InitializeRelationField {
                schema,
                relation,
                from,
                to,
                contract,
                field_path,
            },
            Scope::InitializeRelationField {
                schema: other_schema,
                relation: other_relation,
                from: other_from,
                to: other_to,
                contract: other_contract,
                field_path: other_path,
            },
        ) => (schema, relation, from, to, contract.key(), field_path).cmp(&(
            other_schema,
            other_relation,
            other_from,
            other_to,
            other_contract.key(),
            other_path,
        )),
        (Scope::LinkRelation { .. }, Scope::LinkRelation { .. })
        | (Scope::UnlinkRelation { .. }, Scope::UnlinkRelation { .. }) => {
            relation_order(left, right)
        }
        _ => Ordering::Equal,
    }
}

fn write_field_order(left: &Scope, right: &Scope) -> Ordering {
    let Scope::WriteField {
        schema,
        entity,
        contract,
        field_path,
    } = left
    else {
        return Ordering::Equal;
    };
    let Scope::WriteField {
        schema: other_schema,
        entity: other_entity,
        contract: other_contract,
        field_path: other_path,
    } = right
    else {
        return Ordering::Equal;
    };
    (schema, entity, contract.key().as_str(), field_path).cmp(&(
        other_schema,
        other_entity,
        other_contract.key().as_str(),
        other_path,
    ))
}

fn relation_order(left: &Scope, right: &Scope) -> Ordering {
    relation_fields(left).cmp(&relation_fields(right))
}

fn relation_fields(scope: &Scope) -> (&str, &str, &str, &str) {
    match scope {
        Scope::LinkRelation {
            schema,
            relation,
            from,
            to,
        }
        | Scope::UnlinkRelation {
            schema,
            relation,
            from,
            to,
        } => (schema, relation, from, to),
        _ => ("", "", "", ""),
    }
}

fn tagged_entity(
    output: &mut dyn BinaryEncodingSink,
    tag: u16,
    schema: &str,
    entity: &str,
) -> Result<(), Denial> {
    output.u16(tag)?;
    output.text(schema)?;
    output.text(entity)
}

enum RelationTouch {
    Link,
    Unlink,
}

fn decode_relation(input: &mut BinaryInput<'_>, touch: RelationTouch) -> Result<Scope, Denial> {
    let schema = input.text()?.to_owned();
    let relation = input.text()?.to_owned();
    let from = input.text()?.to_owned();
    let to = input.text()?.to_owned();
    Ok(match touch {
        RelationTouch::Link => Scope::LinkRelation {
            schema,
            relation,
            from,
            to,
        },
        RelationTouch::Unlink => Scope::UnlinkRelation {
            schema,
            relation,
            from,
            to,
        },
    })
}

const fn rank(scope: &Scope) -> u8 {
    match scope {
        Scope::CreateEntity { .. } => 1,
        Scope::DeleteEntity { .. } => 2,
        Scope::WriteField { .. } => 3,
        Scope::LinkRelation { .. } => 4,
        Scope::UnlinkRelation { .. } => 5,
        Scope::InitializeRelationField { .. } => 6,
    }
}
