use worth_foundational::facade::CanonicalizationRuleVersion;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_installation::facade::{
    WorthQueryArtifactContentIdentityContract as ContentIdentity,
    WorthQueryArtifactEvidenceContract as Evidence,
    WorthQueryArtifactFamilyIdentity as FamilyIdentity,
    WorthQueryArtifactOwnershipContract as Ownership,
    WorthQueryArtifactProtocolVersion as ProtocolVersion,
    WorthQueryArtifactSchemaVersion as SchemaVersion,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

pub(super) fn write_family(
    output: &mut dyn BinaryEncodingSink,
    family: &FamilyIdentity,
    schema_version: SchemaVersion,
    protocol_version: ProtocolVersion,
) -> Result<(), Denial> {
    output.text(family.as_str())?;
    output.u32(schema_version.get())?;
    output.u32(protocol_version.get())
}

pub(super) fn decode_family(
    input: &mut BinaryInput<'_>,
) -> Result<(FamilyIdentity, SchemaVersion, ProtocolVersion), Denial> {
    Ok((
        FamilyIdentity::from_untrusted_string(ARCHIVE_DECODE, input.text()?.to_owned()),
        SchemaVersion::new(input.u32()?),
        ProtocolVersion::new(input.u32()?),
    ))
}

pub(super) fn write_content_identity(
    output: &mut dyn BinaryEncodingSink,
    contract: &ContentIdentity,
) -> Result<(), Denial> {
    match contract {
        ContentIdentity::OwnerCanonicalProjection {
            projection_family,
            rule_version,
        } => {
            output.u16(1)?;
            output.text(projection_family)?;
            output.text(rule_version.as_str())
        }
        ContentIdentity::CallerDigestDefined => output.u16(2),
    }
}

pub(super) fn decode_content_identity(
    input: &mut BinaryInput<'_>,
) -> Result<ContentIdentity, Denial> {
    match input.u16()? {
        1 => Ok(ContentIdentity::OwnerCanonicalProjection {
            projection_family: input.text()?.to_owned(),
            rule_version: CanonicalizationRuleVersion::new(input.text()?.to_owned())
                .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?,
        }),
        2 => Ok(ContentIdentity::CallerDigestDefined),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

pub(super) fn write_ownership(
    output: &mut dyn BinaryEncodingSink,
    contract: &Ownership,
) -> Result<(), Denial> {
    match contract {
        Ownership::NotDeclared => output.u16(1),
        Ownership::DomainPayload {
            payload_owner,
            provider_family,
        } => {
            output.u16(2)?;
            output.text(payload_owner)?;
            output.text(provider_family)
        }
    }
}

pub(super) fn decode_ownership(input: &mut BinaryInput<'_>) -> Result<Ownership, Denial> {
    match input.u16()? {
        1 => Ok(Ownership::NotDeclared),
        2 => Ok(Ownership::DomainPayload {
            payload_owner: input.text()?.to_owned(),
            provider_family: input.text()?.to_owned(),
        }),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

pub(super) fn write_evidence(
    output: &mut dyn BinaryEncodingSink,
    contract: &Evidence,
) -> Result<(), Denial> {
    output.text(contract.basis_family())?;
    output.text(contract.provenance_family())?;
    output.text(contract.dependency_family())?;
    output.text(contract.invalidation_family())?;
    output.text(contract.equivalence_family())
}

pub(super) fn decode_evidence(input: &mut BinaryInput<'_>) -> Result<Evidence, Denial> {
    Ok(Evidence::new(
        input.text()?.to_owned(),
        input.text()?.to_owned(),
        input.text()?.to_owned(),
        input.text()?.to_owned(),
        input.text()?.to_owned(),
    ))
}
