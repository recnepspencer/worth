use worth_foundational::facade::{
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact,
    CanonicalBasisSequence, CanonicalBasisValue, CanonicalFloatWidth, CanonicalIntegerWidth,
    InternedString, Symbol,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BridgeCanonicalBasisTextError {
    UnsupportedDomain,
    UnsupportedEntryKind,
}

impl std::fmt::Display for BridgeCanonicalBasisTextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedDomain => {
                write!(f, "unsupported foundational canonical basis domain")
            }
            Self::UnsupportedEntryKind => {
                write!(f, "unsupported foundational canonical basis entry kind")
            }
        }
    }
}

impl std::error::Error for BridgeCanonicalBasisTextError {}

pub(crate) fn canonical_basis_ready_text(
    ready: &CanonicalBasisReadyArtifact,
) -> Result<String, BridgeCanonicalBasisTextError> {
    let sequence = ready_canonical_basis_sequence(ready);
    let entries = sequence
        .entries()
        .iter()
        .map(canonical_basis_entry_text)
        .collect::<Result<Vec<_>, _>>()?
        .join(";");

    Ok(format!(
        "version={};domain={};entries=[{}]",
        sequence.version().as_str(),
        canonical_basis_domain_text(sequence)?,
        entries,
    ))
}

fn ready_canonical_basis_sequence(ready: &CanonicalBasisReadyArtifact) -> &CanonicalBasisSequence {
    ready.payload()
}

fn canonical_basis_domain_text(
    sequence: &CanonicalBasisSequence,
) -> Result<String, BridgeCanonicalBasisTextError> {
    Ok(match sequence.domain() {
        worth_foundational::facade::CanonicalBasisDomain::Value => "value".to_string(),
        worth_foundational::facade::CanonicalBasisDomain::AspectContract => {
            "aspect-contract".to_string()
        }
        worth_foundational::facade::CanonicalBasisDomain::AspectMask => "aspect-mask".to_string(),
        worth_foundational::facade::CanonicalBasisDomain::AuthoritativeState => {
            "authoritative-state".to_string()
        }
        worth_foundational::facade::CanonicalBasisDomain::AuthoritativePatch => {
            "authoritative-patch".to_string()
        }
        worth_foundational::facade::CanonicalBasisDomain::Identity => "identity".to_string(),
        worth_foundational::facade::CanonicalBasisDomain::Locator => "locator".to_string(),
        worth_foundational::facade::CanonicalBasisDomain::Profile => "profile".to_string(),
        worth_foundational::facade::CanonicalBasisDomain::Performance => "performance".to_string(),
        worth_foundational::facade::CanonicalBasisDomain::BoundaryArtifact => {
            "boundary-artifact".to_string()
        }
        worth_foundational::facade::CanonicalBasisDomain::Transition => "transition".to_string(),
        worth_foundational::facade::CanonicalBasisDomain::Diagnostic => "diagnostic".to_string(),
        worth_foundational::facade::CanonicalBasisDomain::Future(name) => name.to_string(),
        _ => return Err(BridgeCanonicalBasisTextError::UnsupportedDomain),
    })
}

fn canonical_basis_entry_text(
    entry: &CanonicalBasisEntry,
) -> Result<String, BridgeCanonicalBasisTextError> {
    Ok(format!(
        "locus={},kind={},value={}",
        canonical_basis_locus_text(entry.locus()),
        canonical_basis_entry_kind_text(entry.kind())?,
        canonical_basis_value_text(entry.value())?,
    ))
}

fn canonical_basis_locus_text(locus: &CanonicalBasisLocus) -> String {
    match locus {
        CanonicalBasisLocus::Root => "root".to_string(),
        CanonicalBasisLocus::EntryOrdinal(ordinal) => format!("ordinal:{ordinal}"),
        CanonicalBasisLocus::Aspect(aspect) => format!("aspect:{}", aspect.as_str()),
        CanonicalBasisLocus::AspectField { aspect, path } => format!(
            "aspect-field:{}:{}",
            aspect.as_str(),
            path.fields()
                .iter()
                .map(|field| field.as_str())
                .collect::<Vec<_>>()
                .join(".")
        ),
        CanonicalBasisLocus::Named(value) => format!("named:{}", interned_string_text(value)),
    }
}

fn canonical_basis_entry_kind_text(
    kind: CanonicalBasisEntryKind,
) -> Result<String, BridgeCanonicalBasisTextError> {
    Ok(match kind {
        CanonicalBasisEntryKind::Header => "header".to_string(),
        CanonicalBasisEntryKind::Shape => "shape".to_string(),
        CanonicalBasisEntryKind::Value => "value".to_string(),
        CanonicalBasisEntryKind::Field => "field".to_string(),
        CanonicalBasisEntryKind::Mask => "mask".to_string(),
        CanonicalBasisEntryKind::StateAspect => "state-aspect".to_string(),
        CanonicalBasisEntryKind::PatchOperation => "patch-operation".to_string(),
        CanonicalBasisEntryKind::Identity => "identity".to_string(),
        CanonicalBasisEntryKind::Locator => "locator".to_string(),
        CanonicalBasisEntryKind::Profile => "profile".to_string(),
        CanonicalBasisEntryKind::PerformanceClaim => "performance-claim".to_string(),
        CanonicalBasisEntryKind::PerformanceLayout => "performance-layout".to_string(),
        CanonicalBasisEntryKind::PerformanceCounter => "performance-counter".to_string(),
        CanonicalBasisEntryKind::PerformanceSupport => "performance-support".to_string(),
        CanonicalBasisEntryKind::BoundaryArtifact => "boundary-artifact".to_string(),
        CanonicalBasisEntryKind::BoundaryAttachment => "boundary-attachment".to_string(),
        CanonicalBasisEntryKind::TransitionArtifact => "transition-artifact".to_string(),
        CanonicalBasisEntryKind::TransitionLocator => "transition-locator".to_string(),
        CanonicalBasisEntryKind::DiagnosticBundle => "diagnostic-bundle".to_string(),
        CanonicalBasisEntryKind::DiagnosticRow => "diagnostic-row".to_string(),
        CanonicalBasisEntryKind::DiagnosticGap => "diagnostic-gap".to_string(),
        CanonicalBasisEntryKind::Cost => "cost".to_string(),
        CanonicalBasisEntryKind::Future(name) => name.to_string(),
        _ => return Err(BridgeCanonicalBasisTextError::UnsupportedEntryKind),
    })
}

fn canonical_basis_value_text(
    value: &CanonicalBasisValue,
) -> Result<String, BridgeCanonicalBasisTextError> {
    Ok(match value {
        CanonicalBasisValue::InlineBytes(value) => {
            format!("inline-bytes:{}", hex_octets(value.as_bytes()))
        }
        CanonicalBasisValue::Null => "null".to_string(),
        CanonicalBasisValue::Bool(value) => format!("bool:{value}"),
        CanonicalBasisValue::SignedInteger { width, value } => {
            format!("signed-{}:{value}", integer_width_text(*width))
        }
        CanonicalBasisValue::UnsignedInteger { width, value } => {
            format!("unsigned-{}:{value}", integer_width_text(*width))
        }
        CanonicalBasisValue::FloatBits { width, bits } => {
            format!("float-{}:{bits}", float_width_text(*width))
        }
        CanonicalBasisValue::ExactText(value) => {
            format!("exact-text:{}", interned_string_text(value))
        }
        CanonicalBasisValue::BytesDigest(value) => {
            format!("binary-content-digest:{}", hex_octets(value.bytes()))
        }
        CanonicalBasisValue::DecimalText(value) => {
            format!("decimal-text:{}", interned_string_text(value))
        }
        CanonicalBasisValue::BigIntText(value) => {
            format!("big-int-text:{}", interned_string_text(value))
        }
        CanonicalBasisValue::RationalText {
            numerator,
            denominator,
        } => format!(
            "rational-text:{}/{}",
            interned_string_text(numerator),
            interned_string_text(denominator)
        ),
        CanonicalBasisValue::BytesRefId(value) => format!("binary-content-ref:{value}"),
        CanonicalBasisValue::ContentRefId(value) => format!("content-ref:{value}"),
        CanonicalBasisValue::EntityRef {
            partition_id,
            local_slot,
            generation,
        } => format!("entity-ref:{partition_id}:{local_slot}:{generation}"),
        CanonicalBasisValue::DateDays(value) => format!("date-days:{value}"),
        CanonicalBasisValue::TimeNanos(value) => format!("time-nanos:{value}"),
        CanonicalBasisValue::TimestampMicros(value) => format!("timestamp-micros:{value}"),
        CanonicalBasisValue::TimestampTz {
            utc_micros_since_unix_epoch,
            offset_minutes,
        } => format!("timestamp-tz:{utc_micros_since_unix_epoch}:{offset_minutes}"),
        CanonicalBasisValue::UuidBytes(value) => format!("uuid:{}", hex_octets(value)),
        CanonicalBasisValue::NestedSequence(value) => format!("nested-sequence:{value}"),
    })
}

fn hex_octets(octets: &[u8]) -> String {
    octets.iter().map(|octet| format!("{octet:02x}")).collect()
}

fn integer_width_text(width: CanonicalIntegerWidth) -> &'static str {
    match width {
        CanonicalIntegerWidth::Bits8 => "i8",
        CanonicalIntegerWidth::Bits16 => "i16",
        CanonicalIntegerWidth::Bits32 => "i32",
        CanonicalIntegerWidth::Bits64 => "i64",
        CanonicalIntegerWidth::Bits128 => "i128",
    }
}

fn float_width_text(width: CanonicalFloatWidth) -> &'static str {
    match width {
        CanonicalFloatWidth::Bits32 => "f32",
        CanonicalFloatWidth::Bits64 => "f64",
    }
}

fn interned_string_text(value: &InternedString) -> String {
    match value {
        InternedString::Raw(value) => value.clone(),
        InternedString::Symbol(Symbol(value)) => format!("symbol:{value}"),
    }
}
