use worth_foundational::facade::{AspectContractRevision, AspectIdentity};
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldPresence, ApplicationRelationCardinality,
    ApplicationRelationCrossContextPolicy, ApplicationRelationDeletionPolicy,
    ApplicationRelationEndpoints, ApplicationRelationIntegrity, ApplicationSchemaMember,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::super::{foundational_aspect, foundational_value};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    member: &ApplicationSchemaMember,
) -> Result<(), Denial> {
    match member {
        ApplicationSchemaMember::Entity { entity } => output.text(entity),
        ApplicationSchemaMember::Aspect {
            entity,
            aspect,
            identity,
            revision,
        } => {
            output.text(entity)?;
            output.text(aspect)?;
            output.u64(identity.0)?;
            output.u64(revision.0)
        }
        ApplicationSchemaMember::Field { .. } => write_field(output, member),
        ApplicationSchemaMember::Relation { .. } => write_relation(output, member),
        ApplicationSchemaMember::PrincipalBinding {
            binding,
            mapping_entity,
            identity_aspect,
            identity_field,
            status_aspect,
            status_field,
            target_relation,
            principal_entity,
            principal_identity_aspect,
            principal_identity_field,
            principal_identity_scalar_family,
            principal_identity_value_type,
        } => {
            for value in [
                binding,
                mapping_entity,
                identity_aspect,
                identity_field,
                status_aspect,
                status_field,
                target_relation,
                principal_entity,
                principal_identity_aspect,
                principal_identity_field,
            ] {
                output.text(value)?;
            }
            foundational_aspect::write_scalar_type(output, *principal_identity_scalar_family)?;
            output.text(principal_identity_value_type)
        }
        _ => unreachable!("schema member dispatch is exhaustive"),
    }
}

fn write_field(
    output: &mut dyn BinaryEncodingSink,
    member: &ApplicationSchemaMember,
) -> Result<(), Denial> {
    let ApplicationSchemaMember::Field {
        entity,
        aspect,
        field,
        presence,
        scalar_family,
        value_type,
        unit,
        frame,
        writable,
        equality_queryable,
    } = member
    else {
        unreachable!()
    };
    output.text(entity)?;
    output.text(aspect)?;
    output.text(field)?;
    write_presence(output, *presence)?;
    foundational_aspect::write_scalar_type(output, *scalar_family)?;
    output.text(value_type)?;
    super::super::wire_vocabulary::write_optional(output, unit.as_ref(), |output, unit| {
        output.text(unit)
    })?;
    if let Some(frame) = frame {
        output.text(frame)?;
    }
    foundational_value::write_bool(output, *writable)?;
    foundational_value::write_bool(output, *equality_queryable)
}

fn write_relation(
    output: &mut dyn BinaryEncodingSink,
    member: &ApplicationSchemaMember,
) -> Result<(), Denial> {
    let ApplicationSchemaMember::Relation {
        relation,
        from,
        to,
        integrity,
    } = member
    else {
        unreachable!()
    };
    output.text(relation)?;
    output.text(from)?;
    output.text(to)?;
    if *integrity == ApplicationRelationIntegrity::same_context_unbounded_retain_dangling() {
        Ok(())
    } else {
        write_relation_integrity(output, *integrity)
    }
}

pub(super) fn decode(
    tag: u16,
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationSchemaMember, Denial> {
    Ok(match tag {
        1 => ApplicationSchemaMember::Entity {
            entity: input.text()?.to_owned(),
        },
        2 => ApplicationSchemaMember::Aspect {
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            identity: AspectIdentity(input.u64()?),
            revision: AspectContractRevision(input.u64()?),
        },
        3 | 25 => ApplicationSchemaMember::Field {
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            field: input.text()?.to_owned(),
            presence: decode_presence(input)?,
            scalar_family: foundational_aspect::decode_scalar_type(input)?,
            value_type: input.text()?.to_owned(),
            unit: super::super::wire_vocabulary::decode_optional(input, |input| {
                Ok(input.text()?.to_owned())
            })?,
            frame: (tag == 25)
                .then(|| input.text().map(str::to_owned))
                .transpose()?,
            writable: foundational_value::decode_bool(input)?,
            equality_queryable: foundational_value::decode_bool(input)?,
        },
        4 | 26 => ApplicationSchemaMember::Relation {
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
            integrity: if tag == 26 {
                decode_relation_integrity(input)?
            } else {
                ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            },
        },
        5 => ApplicationSchemaMember::PrincipalBinding {
            binding: input.text()?.to_owned(),
            mapping_entity: input.text()?.to_owned(),
            identity_aspect: input.text()?.to_owned(),
            identity_field: input.text()?.to_owned(),
            status_aspect: input.text()?.to_owned(),
            status_field: input.text()?.to_owned(),
            target_relation: input.text()?.to_owned(),
            principal_entity: input.text()?.to_owned(),
            principal_identity_aspect: input.text()?.to_owned(),
            principal_identity_field: input.text()?.to_owned(),
            principal_identity_scalar_family: foundational_aspect::decode_scalar_type(input)?,
            principal_identity_value_type: input.text()?.to_owned(),
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}

fn write_relation_integrity(
    output: &mut dyn BinaryEncodingSink,
    integrity: ApplicationRelationIntegrity,
) -> Result<(), Denial> {
    output.u16(match integrity.endpoints.cross_context_policy {
        ApplicationRelationCrossContextPolicy::AllowExplicit => 1,
        ApplicationRelationCrossContextPolicy::SchemaControlled => 2,
        ApplicationRelationCrossContextPolicy::Forbid => 3,
    })?;
    foundational_value::write_bool(output, integrity.endpoints.self_edges_allowed)?;
    for value in [
        integrity.cardinality.source_min,
        integrity.cardinality.source_max,
        integrity.cardinality.target_min,
        integrity.cardinality.target_max,
        integrity.cardinality.pair_min,
        integrity.cardinality.pair_max,
    ] {
        super::super::wire_vocabulary::write_optional(output, value.as_ref(), |output, value| {
            output.u64(*value)
        })?;
    }
    output.u16(match integrity.deletion {
        ApplicationRelationDeletionPolicy::RetainDanglingForAudit => 1,
        ApplicationRelationDeletionPolicy::CascadeDeleteRelations => 2,
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations => 3,
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit => 4,
        ApplicationRelationDeletionPolicy::RequireRelationRetirement => 5,
    })
}

fn decode_relation_integrity(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationRelationIntegrity, Denial> {
    let cross_context_policy = match input.u16()? {
        1 => ApplicationRelationCrossContextPolicy::AllowExplicit,
        2 => ApplicationRelationCrossContextPolicy::SchemaControlled,
        3 => ApplicationRelationCrossContextPolicy::Forbid,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let self_edges_allowed = foundational_value::decode_bool(input)?;
    let mut next_bound =
        || super::super::wire_vocabulary::decode_optional(input, |input| input.u64());
    let cardinality = ApplicationRelationCardinality::new(
        next_bound()?,
        next_bound()?,
        next_bound()?,
        next_bound()?,
        next_bound()?,
        next_bound()?,
    );
    let deletion = match input.u16()? {
        1 => ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
        2 => ApplicationRelationDeletionPolicy::CascadeDeleteRelations,
        3 => ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations,
        4 => ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
        5 => ApplicationRelationDeletionPolicy::RequireRelationRetirement,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(self_edges_allowed, cross_context_policy),
        cardinality,
        deletion,
    ))
}

pub(super) fn write_presence(
    output: &mut dyn BinaryEncodingSink,
    value: ApplicationFieldPresence,
) -> Result<(), Denial> {
    output.u16(match value {
        ApplicationFieldPresence::Required => 1,
        ApplicationFieldPresence::Optional => 2,
    })
}

pub(super) fn decode_presence(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationFieldPresence, Denial> {
    match input.u16()? {
        1 => Ok(ApplicationFieldPresence::Required),
        2 => Ok(ApplicationFieldPresence::Optional),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
