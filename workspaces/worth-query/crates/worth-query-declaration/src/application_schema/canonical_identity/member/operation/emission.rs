use crate::application_schema::{
    canonical_basis::ApplicationSchemaCanonicalBasis, ApplicationOperationBodyInputEmission,
};

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    emission: &ApplicationOperationBodyInputEmission,
) {
    basis.text(
        format!("{prefix}.emission.source"),
        "exact-operation-input-v1",
    );
    basis.text(format!("{prefix}.emission.effect"), &emission.effect);
    basis.text(
        format!("{prefix}.emission.input-binding"),
        emission.input_binding.as_str(),
    );
    basis.text(
        format!("{prefix}.emission.payload-binding"),
        emission.payload_binding.as_str(),
    );
    basis.text(
        format!("{prefix}.emission.protocol"),
        emission.protocol.identity().as_str(),
    );
    basis.u64(
        format!("{prefix}.emission.protocol-version"),
        u64::from(emission.protocol.version().get()),
    );
    basis.u64(
        format!("{prefix}.emission.maximum-bytes"),
        emission.maximum_payload_bytes,
    );
    basis.text(
        format!("{prefix}.emission.condition"),
        &emission
            .condition
            .as_ref()
            .map(|condition| condition.canonical_record())
            .unwrap_or_else(|| "unconditional".into()),
    );
}
