use worth_foundational::facade::{AspectContractRevision, AspectIdentity};
use worth_query_declaration::facade::application_schema::{
    ApplicationRelationIntegrity, ApplicationSchemaMember,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::super::{foundational_aspect, foundational_value};
use crate::record::decode_budget::RecordDecodeAttempt;
mod relation_integrity;

pub(super) fn require_supported_field_codec(
    member: &ApplicationSchemaMember,
) -> Result<(), Denial> {
    if matches!(
        member,
        ApplicationSchemaMember::Field {
            canonical: Some(_),
            ..
        }
    ) {
        return Err(Denial::new(Kind::UnsupportedCanonicalValueContract));
    }
    Ok(())
}

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
            absence,
        } => {
            output.text(entity)?;
            output.text(aspect)?;
            output.u64(identity.0)?;
            output.u64(revision.0)?;
            super::super::super::foundational_aspect::write_absence(output, *absence)
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
        canonical: _,
        value_type,
        unit,
        frame,
        writable,
        equality_queryable,
        unique,
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
    foundational_value::write_bool(output, *equality_queryable)?;
    foundational_value::write_bool(output, *unique)
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
    if integrity == &ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().into() {
        Ok(())
    } else {
        relation_integrity::write(output, integrity)
    }
}

pub(super) fn decode(
    tag: u16,
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
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
            absence: super::super::super::foundational_aspect::decode_absence(input)?,
        },
        3 | 25 => ApplicationSchemaMember::Field {
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            field: input.text()?.to_owned(),
            presence: decode_presence(input)?,
            scalar_family: foundational_aspect::decode_scalar_type(input)?,
            canonical: None,
            value_type: input.text()?.to_owned(),
            unit: super::super::wire_vocabulary::decode_optional(input, |input| {
                Ok(input.text()?.to_owned())
            })?,
            frame: (tag == 25)
                .then(|| input.text().map(str::to_owned))
                .transpose()?,
            writable: foundational_value::decode_bool(input)?,
            equality_queryable: foundational_value::decode_bool(input)?,
            unique: foundational_value::decode_bool(input)?,
        },
        4 | 26 => ApplicationSchemaMember::Relation {
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
            integrity: if tag == 26 {
                relation_integrity::decode(input, budget)?
            } else {
                ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().into()
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

pub(super) use super::super::wire_vocabulary::{decode_presence, write_presence};
