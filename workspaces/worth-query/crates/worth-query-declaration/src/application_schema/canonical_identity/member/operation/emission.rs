use crate::application_schema::{
    canonical_basis::ApplicationSchemaCanonicalBasis, ApplicationOperationBodyExternalEmission,
};

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    emission: &ApplicationOperationBodyExternalEmission,
) {
    match &emission.source {
        crate::application_schema::ApplicationOperationBodyExternalPayloadSource::Evaluation { mutation_binding, query } => {
            basis.text(format!("{prefix}.emission.source"), "native-body-evaluation-v1");
            basis.text(format!("{prefix}.emission.source-binding"), mutation_binding);
            basis.extend_embedded(query.canonical_basis().basis(), &format!("{prefix}.emission.query"), worth_foundational::facade::CanonicalBasisEntryKind::Identity);
        }
        crate::application_schema::ApplicationOperationBodyExternalPayloadSource::OperationInput => basis.text(format!("{prefix}.emission.source"), "exact-operation-input-v1"),
        crate::application_schema::ApplicationOperationBodyExternalPayloadSource::QueryContext { mutation_binding, query } => {
            basis.text(format!("{prefix}.emission.source"), "exact-native-query-context-v1");
            basis.text(format!("{prefix}.emission.source-binding"), mutation_binding);
            basis.extend_embedded(query.canonical_basis().basis(), &format!("{prefix}.emission.query"), worth_foundational::facade::CanonicalBasisEntryKind::Identity);
        }
    }
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
