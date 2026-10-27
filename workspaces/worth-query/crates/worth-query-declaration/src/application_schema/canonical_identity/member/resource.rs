use crate::application_resource::{
    ApplicationResourceAction, ApplicationResourceValueBindingReference,
    ErasedApplicationResourceDefinition,
};
use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use worth_foundational::facade::CanonicalBasisEntryKind;

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    definition: &ErasedApplicationResourceDefinition,
) {
    let parts = definition.parts();
    basis.text(format!("{prefix}.kind"), "application-resource");
    basis.text(format!("{prefix}.name"), &parts.name);
    basis.text(
        format!("{prefix}.resource-type"),
        parts.resource_type.as_str(),
    );
    basis.text(
        format!("{prefix}.provider-role"),
        parts.provider_role.as_str(),
    );
    for (name, value) in parts.access.canonical_fields() {
        basis.text(format!("{prefix}.access.{name}"), &value);
    }
    for (name, value) in [
        ("input", &parts.input),
        ("result", &parts.result),
        ("failure", &parts.failure),
    ] {
        append_value(basis, &format!("{prefix}.{name}"), value);
    }
    basis.u32(
        format!("{prefix}.retry.maximum-attempts"),
        parts.profile.retry_max_attempts,
    );
    basis.u64(
        format!("{prefix}.retry.delay-ms"),
        parts.profile.retry_delay_milliseconds,
    );
    basis.u64(
        format!("{prefix}.timeout-ms"),
        parts.profile.timeout_milliseconds,
    );
    basis.u32(format!("{prefix}.limits.live"), parts.limits.maximum_live);
    basis.u32(
        format!("{prefix}.limits.results"),
        parts.limits.maximum_retained_results,
    );
    basis.u32(
        format!("{prefix}.limits.ingress"),
        parts.limits.ingress_slots_per_request,
    );
    basis.u64(
        format!("{prefix}.limits.input-bytes"),
        parts.limits.maximum_input_bytes,
    );
    basis.u64(
        format!("{prefix}.limits.frame-bytes"),
        parts.limits.maximum_frame_bytes,
    );
    for action in ApplicationResourceAction::ALL {
        let requirement = parts.authorization.requirement(action);
        let action = action.name();
        basis.optional_text(
            format!("{prefix}.action.{action}.ability"),
            requirement.ability(),
        );
        basis.optional_text(
            format!("{prefix}.action.{action}.scope"),
            requirement.scope_entity(),
        );
    }
    basis.extend_embedded(
        parts.disclosure.canonical_basis().basis(),
        &format!("{prefix}.disclosure"),
        CanonicalBasisEntryKind::Identity,
    );
}

fn append_value(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    reference: &ApplicationResourceValueBindingReference,
) {
    let value = reference.parts();
    basis.text(format!("{prefix}.identity"), value.identity.as_str());
    basis.text(
        format!("{prefix}.family"),
        value.scalar_family.canonical_name(),
    );
    basis.optional_text(format!("{prefix}.unit"), value.unit.as_deref());
    basis.optional_text(format!("{prefix}.frame"), value.frame.as_deref());
    basis.bool(
        format!("{prefix}.admitted-decode"),
        value.supports_admitted_decode,
    );
    basis.bool(
        format!("{prefix}.native-admission"),
        value.native_input_admission.is_some(),
    );
    if let Some(native) = value.native_input_admission {
        basis.u64(format!("{prefix}.native.work-base"), native.work_base());
        basis.u64(
            format!("{prefix}.native.work-per-byte"),
            native.work_per_byte(),
        );
        basis.u64(
            format!("{prefix}.native.memory-base"),
            native.peak_memory_base(),
        );
        basis.u64(
            format!("{prefix}.native.memory-per-byte"),
            native.peak_memory_per_byte(),
        );
    }
    basis.bool(format!("{prefix}.canonical"), value.canonical.is_some());
    if let Some(canonical) = &value.canonical {
        let canonical = canonical.parts();
        basis.text(format!("{prefix}.codec.protocol"), &canonical.protocol);
        basis.text(format!("{prefix}.codec.shape"), &canonical.shape);
        basis.bool(
            format!("{prefix}.codec.admission"),
            canonical.supports_admission,
        );
        for (name, maximum) in [
            ("wire", canonical.maximum_wire_bytes),
            ("string", canonical.maximum_string_bytes),
            ("sequence", canonical.maximum_sequence_length),
            ("depth", canonical.maximum_depth),
        ] {
            match maximum {
                Some(maximum) => basis.u64(format!("{prefix}.codec.bounds.{name}"), maximum),
                None => basis.text(format!("{prefix}.codec.bounds.{name}"), "unrestricted"),
            }
        }
    }
}
