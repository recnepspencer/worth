use worth_foundational::facade::{
    canonicalization, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisValue, CanonicalFloatWidth, CanonicalIntegerWidth, CanonicalizationRuleVersion,
    InternedString,
};
use worth_proof::TransitionOutcome;

use crate::canonical_basis_ready_sequence::canonical_basis_ready_sequence;
use crate::identity::data::KindId;
use crate::schema::data::{DeclaredAspectContractBinding, SchemaRegistryError};

use super::schema_plan_terms::RevisionHasher;

const SCHEMA_ASPECT_PLAN_REVISION_BASIS: &str = "worth-relational.schema.aspect-plan-revision.v1";

pub(super) fn mix_foundational_contract_basis(
    revision: &mut RevisionHasher,
    kind_id: KindId,
    aspect: &DeclaredAspectContractBinding,
) -> Result<(), SchemaRegistryError> {
    let Some(version) = CanonicalizationRuleVersion::new(SCHEMA_ASPECT_PLAN_REVISION_BASIS) else {
        return Err(SchemaRegistryError::invalid_aspect_declaration(
            kind_id,
            "schema aspect plan revision canonicalization rule version is invalid",
        ));
    };

    let ready = match canonicalization()
        .basis()
        .at(version)
        .from_contract(aspect.contract.clone())
    {
        TransitionOutcome::Success(ready) => ready,
        TransitionOutcome::Denied(denial) => {
            return Err(SchemaRegistryError::invalid_aspect_declaration(
                kind_id,
                format!(
                    "foundational contract canonical-basis preparation denied aspect `{}`: {:?}",
                    aspect.aspect_key().as_str(),
                    denial
                ),
            ));
        }
    };

    revision.mix_text("foundational_contract_basis");
    let ready_sequence = canonical_basis_ready_sequence(&ready);
    revision.mix_text(ready_sequence.version().as_str());
    for entry in ready_sequence.entries() {
        mix_canonical_basis_entry(revision, entry);
    }
    Ok(())
}

fn mix_canonical_basis_entry(revision: &mut RevisionHasher, entry: &CanonicalBasisEntry) {
    revision.mix_text("basis_entry");
    mix_canonical_entry_kind(revision, entry.kind());
    mix_canonical_locus(revision, entry.locus());
    mix_canonical_value(revision, entry.value());
}

fn mix_canonical_locus(revision: &mut RevisionHasher, locus: &CanonicalBasisLocus) {
    match locus {
        CanonicalBasisLocus::Root => revision.mix_u8(0),
        CanonicalBasisLocus::EntryOrdinal(value) => {
            revision.mix_u8(1);
            revision.mix_u32(*value);
        }
        CanonicalBasisLocus::Aspect(aspect_key) => {
            revision.mix_u8(2);
            revision.mix_aspect_key(aspect_key);
        }
        CanonicalBasisLocus::AspectField { aspect, path } => {
            revision.mix_u8(3);
            revision.mix_aspect_key(aspect);
            revision.mix_field_path(path.fields());
        }
        CanonicalBasisLocus::Named(value) => {
            revision.mix_u8(4);
            mix_interned_string(revision, value);
        }
    }
}

fn mix_canonical_value(revision: &mut RevisionHasher, value: &CanonicalBasisValue) {
    match value {
        CanonicalBasisValue::InlineBytes(value) => {
            revision.mix_u8(19);
            revision.mix_bytes(value.as_bytes());
        }
        CanonicalBasisValue::Null => revision.mix_u8(0),
        CanonicalBasisValue::Bool(value) => {
            revision.mix_u8(1);
            revision.mix_bool(*value);
        }
        CanonicalBasisValue::SignedInteger { width, value } => {
            revision.mix_u8(2);
            revision.mix_u8(canonical_integer_width_tag(*width));
            revision.mix_i128(*value);
        }
        CanonicalBasisValue::UnsignedInteger { width, value } => {
            revision.mix_u8(3);
            revision.mix_u8(canonical_integer_width_tag(*width));
            revision.mix_u128(*value);
        }
        CanonicalBasisValue::FloatBits { width, bits } => {
            revision.mix_u8(4);
            revision.mix_u8(canonical_float_width_tag(*width));
            revision.mix_u64(*bits);
        }
        CanonicalBasisValue::ExactText(value) => {
            revision.mix_u8(5);
            mix_interned_string(revision, value);
        }
        CanonicalBasisValue::BytesDigest(value) => {
            revision.mix_u8(6);
            revision.mix_bytes(value.bytes());
        }
        CanonicalBasisValue::DecimalText(value) => {
            revision.mix_u8(7);
            mix_interned_string(revision, value);
        }
        CanonicalBasisValue::BigIntText(value) => {
            revision.mix_u8(8);
            mix_interned_string(revision, value);
        }
        CanonicalBasisValue::RationalText {
            numerator,
            denominator,
        } => {
            revision.mix_u8(9);
            mix_interned_string(revision, numerator);
            mix_interned_string(revision, denominator);
        }
        CanonicalBasisValue::BytesRefId(value) => {
            revision.mix_u8(10);
            revision.mix_u64(*value);
        }
        CanonicalBasisValue::ContentRefId(value) => {
            revision.mix_u8(11);
            revision.mix_bytes(value);
        }
        CanonicalBasisValue::EntityRef {
            partition_id,
            local_slot,
            generation,
        } => {
            revision.mix_u8(12);
            revision.mix_u32(*partition_id);
            revision.mix_u64(*local_slot);
            revision.mix_u32(*generation);
        }
        CanonicalBasisValue::DateDays(value) => {
            revision.mix_u8(13);
            revision.mix_i32(*value);
        }
        CanonicalBasisValue::TimeNanos(value) => {
            revision.mix_u8(14);
            revision.mix_u64(*value);
        }
        CanonicalBasisValue::TimestampMicros(value) => {
            revision.mix_u8(15);
            revision.mix_i64(*value);
        }
        CanonicalBasisValue::TimestampTz {
            utc_micros_since_unix_epoch,
            offset_minutes,
        } => {
            revision.mix_u8(16);
            revision.mix_i64(*utc_micros_since_unix_epoch);
            revision.mix_i32(*offset_minutes);
        }
        CanonicalBasisValue::UuidBytes(value) => {
            revision.mix_u8(17);
            revision.mix_bytes(value);
        }
        CanonicalBasisValue::NestedSequence(value) => {
            revision.mix_u8(18);
            revision.mix_u32(*value);
        }
    }
}

fn mix_interned_string(revision: &mut RevisionHasher, value: &InternedString) {
    match value {
        InternedString::Raw(value) => {
            revision.mix_u8(0);
            revision.mix_text(value);
        }
        InternedString::Symbol(symbol) => {
            revision.mix_u8(1);
            revision.mix_u32(symbol.0);
        }
    }
}

fn mix_canonical_entry_kind(revision: &mut RevisionHasher, kind: CanonicalBasisEntryKind) {
    revision.mix_u8(canonical_entry_kind_tag(kind));
    if let CanonicalBasisEntryKind::Future(label) = kind {
        revision.mix_text(label);
    }
}

fn canonical_entry_kind_tag(kind: CanonicalBasisEntryKind) -> u8 {
    match kind {
        CanonicalBasisEntryKind::Header => 0,
        CanonicalBasisEntryKind::Shape => 1,
        CanonicalBasisEntryKind::Value => 2,
        CanonicalBasisEntryKind::Field => 3,
        CanonicalBasisEntryKind::Mask => 4,
        CanonicalBasisEntryKind::StateAspect => 5,
        CanonicalBasisEntryKind::PatchOperation => 6,
        CanonicalBasisEntryKind::Identity => 7,
        CanonicalBasisEntryKind::Locator => 8,
        CanonicalBasisEntryKind::Profile => 9,
        CanonicalBasisEntryKind::PerformanceClaim => 10,
        CanonicalBasisEntryKind::PerformanceLayout => 11,
        CanonicalBasisEntryKind::PerformanceCounter => 12,
        CanonicalBasisEntryKind::PerformanceSupport => 13,
        CanonicalBasisEntryKind::BoundaryArtifact => 14,
        CanonicalBasisEntryKind::BoundaryAttachment => 15,
        CanonicalBasisEntryKind::TransitionArtifact => 16,
        CanonicalBasisEntryKind::TransitionLocator => 17,
        CanonicalBasisEntryKind::DiagnosticBundle => 18,
        CanonicalBasisEntryKind::DiagnosticRow => 19,
        CanonicalBasisEntryKind::DiagnosticGap => 20,
        CanonicalBasisEntryKind::CompatibilityOrigin => 21,
        CanonicalBasisEntryKind::Cost => 22,
        CanonicalBasisEntryKind::Future(_) => 255,
    }
}

fn canonical_integer_width_tag(width: CanonicalIntegerWidth) -> u8 {
    match width {
        CanonicalIntegerWidth::Bits8 => 0,
        CanonicalIntegerWidth::Bits16 => 1,
        CanonicalIntegerWidth::Bits32 => 2,
        CanonicalIntegerWidth::Bits64 => 3,
        CanonicalIntegerWidth::Bits128 => 4,
    }
}

fn canonical_float_width_tag(width: CanonicalFloatWidth) -> u8 {
    match width {
        CanonicalFloatWidth::Bits32 => 0,
        CanonicalFloatWidth::Bits64 => 1,
    }
}
