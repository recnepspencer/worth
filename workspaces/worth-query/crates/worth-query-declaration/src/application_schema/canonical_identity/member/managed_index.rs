//! The schema's sole canonical projection of managed-index meaning.
use crate::application_query::ErasedApplicationManagedIndexDefinition;
use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    definition: &ErasedApplicationManagedIndexDefinition,
) {
    let parts = definition.parts();
    basis.text(format!("{prefix}.kind"), "managed-index");
    basis.u64(format!("{prefix}.version"), parts.semantic_version.into());
    basis.text(format!("{prefix}.name"), &parts.name);
    basis.text(format!("{prefix}.provider"), &parts.provider);
    basis.u32(format!("{prefix}.provider-version"), parts.provider_version);
    basis.text(format!("{prefix}.entity"), &parts.entity);
    for (name, value) in parts.source.binding.canonical_fields() {
        basis.text(format!("{prefix}.source.{name}"), value);
    }
    basis.text(
        format!("{prefix}.source.scope-policy"),
        "same-admitted-scope",
    );
    basis.text(format!("{prefix}.source.parameter-policy"), "none");
    basis.text(format!("{prefix}.source.posture"), "public-committed");
    basis.text(format!("{prefix}.reconstruction"), "complete-source");
    basis.text(
        format!("{prefix}.dependencies"),
        "conservative-complete-source",
    );
    basis.text(format!("{prefix}.determinism"), "deterministic");
    basis.text(
        format!("{prefix}.candidate-order"),
        "canonical-source-identity",
    );
    let probe = parts.probe.parts();
    basis.text(format!("{prefix}.probe.binding"), probe.binding.as_str());
    basis.text(
        format!("{prefix}.probe.family"),
        probe.scalar_family.canonical_name(),
    );
    basis.optional_text(format!("{prefix}.probe.unit"), probe.unit.as_deref());
    basis.optional_text(format!("{prefix}.probe.frame"), probe.frame.as_deref());
    basis.bool(
        format!("{prefix}.probe.admission"),
        probe.supports_admitted_decode,
    );
    basis.bool(
        format!("{prefix}.probe.canonical"),
        probe.canonical.is_some(),
    );
    if let Some(canonical) = &probe.canonical {
        let codec = canonical.parts();
        basis.text(format!("{prefix}.probe.codec.protocol"), &codec.protocol);
        basis.text(format!("{prefix}.probe.codec.shape"), &codec.shape);
        basis.bool(
            format!("{prefix}.probe.codec.admission"),
            codec.supports_admission,
        );
        for (name, maximum) in [
            ("wire", codec.maximum_wire_bytes),
            ("string", codec.maximum_string_bytes),
            ("sequence", codec.maximum_sequence_length),
            ("depth", codec.maximum_depth),
        ] {
            match maximum {
                Some(value) => basis.u64(format!("{prefix}.probe.codec.{name}"), value),
                None => basis.text(format!("{prefix}.probe.codec.{name}"), "none"),
            }
        }
    }
}
