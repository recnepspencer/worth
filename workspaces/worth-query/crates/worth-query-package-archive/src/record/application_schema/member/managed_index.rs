//! Member48: v1 Opaque; v2 structure; v3 membership. No executable root is transported.
use super::*;
use crate::record::application_schema::wire_vocabulary::{
    decode_optional, decode_type_identity, write_optional, write_type_identity,
};
use crate::record::application_schema::{canonical_value_reference, query_binding_reference};
use worth_query_declaration::facade::application_query::{
    ApplicationManagedIndexMembership as Membership, ApplicationManagedIndexProbeMeaning,
    ApplicationManagedIndexSourceMeaning, ApplicationManagedIndexSourceParameters,
    ApplicationManagedIndexSourceScope, ApplicationManagedIndexSourceStructure as Structure,
    ErasedApplicationManagedIndexDefinition as Definition,
    WorthQueryPortableManagedIndexParts as Parts,
    WorthQueryPortableManagedIndexProbeParts as Probe,
};

pub(super) fn require_shape(member: &ApplicationSchemaMember) -> Result<(), Denial> {
    if let ApplicationSchemaMember::ManagedIndex { definition } = member {
        definition
            .validate()
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    }
    Ok(())
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &Definition,
) -> Result<(), Denial> {
    definition
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let parts = definition.parts();
    output.u16(parts.semantic_version)?;
    output.text(&parts.name)?;
    output.text(&parts.provider)?;
    output.u32(parts.provider_version)?;
    if parts.semantic_version >= 2 {
        output.u16(match parts.structure {
            Structure::Opaque => 0,
            Structure::BinarySourceTree => 1,
        })?;
    }
    if parts.semantic_version >= 3 {
        output.u16(match parts.membership {
            Membership::Candidates => 0,
            Membership::ExhaustiveSourceTree => 1,
            Membership::CheckedSummarySourceTree => 2,
        })?;
    }
    query_binding_reference::write(output, &parts.source.binding)?;
    output.u16(match parts.source.scope {
        ApplicationManagedIndexSourceScope::SameAdmittedScope => 1,
    })?;
    output.u16(match parts.source.parameters {
        ApplicationManagedIndexSourceParameters::NoParameters => 1,
    })?;
    output.text(&parts.entity)?;
    let probe = parts.probe.parts();
    write_type_identity(output, &probe.binding)?;
    crate::record::foundational_aspect::write_scalar_type(output, probe.scalar_family)?;
    write_optional(output, probe.unit.as_ref(), |output, value| {
        output.text(value)
    })?;
    write_optional(output, probe.frame.as_ref(), |output, value| {
        output.text(value)
    })?;
    write_optional(
        output,
        probe.canonical.as_ref(),
        canonical_value_reference::write,
    )?;
    crate::record::foundational_value::write_bool(output, probe.supports_admitted_decode)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    let semantic_version = input.u16()?;
    if !matches!(semantic_version, 1 | 2 | 3) {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    budget.require_nesting_depth(2)?;
    let name = input.text()?.to_owned();
    let provider = input.text()?.to_owned();
    let provider_version = input.u32()?;
    let structure = if semantic_version == 1 {
        Structure::Opaque
    } else {
        match input.u16()? {
            0 => Structure::Opaque,
            1 => Structure::BinarySourceTree,
            _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
        }
    };
    let membership = if semantic_version < 3 {
        Membership::Candidates
    } else {
        match input.u16()? {
            0 => Membership::Candidates,
            1 => Membership::ExhaustiveSourceTree,
            2 => Membership::CheckedSummarySourceTree,
            _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
        }
    };
    let binding = query_binding_reference::decode(input)?;
    let scope = match input.u16()? {
        1 => ApplicationManagedIndexSourceScope::SameAdmittedScope,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let parameters = match input.u16()? {
        1 => ApplicationManagedIndexSourceParameters::NoParameters,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let entity = input.text()?.to_owned();
    let probe = ApplicationManagedIndexProbeMeaning::from_untrusted_parts(Probe {
        binding: decode_type_identity(input)?,
        scalar_family: crate::record::foundational_aspect::decode_scalar_type(input)?,
        unit: decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
        frame: decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
        canonical: decode_optional(input, canonical_value_reference::decode)?,
        supports_admitted_decode: crate::record::foundational_value::decode_bool(input)?,
    });
    let member = ApplicationSchemaMember::ManagedIndex {
        definition: Definition::from_untrusted_parts(Parts {
            semantic_version,
            name,
            provider,
            provider_version,
            structure,
            membership,
            source: ApplicationManagedIndexSourceMeaning {
                binding,
                scope,
                parameters,
            },
            entity,
            probe,
        }),
    };
    require_shape(&member)?;
    Ok(member)
}
