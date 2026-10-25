use serde_json::{json, Value};
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_integrity::*;

pub(super) fn project(outcome: PhysicalIntegrityObservationOutcome) -> Value {
    match outcome {
        PhysicalIntegrityObservationOutcome::Intact(_) => json!({"posture":"intact"}),
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Damaged(
            damage,
        )) => json!({
            "posture":"damaged", "cause":super::vocabulary::cause(damage.cause()),
            "damaged_range":range(damage.damaged_range()), "field":damage.field().map(super::vocabulary::field),
            "blast_radius":super::vocabulary::blast(damage.blast_radius())}),
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Unsupported(
            value,
        )) => json!({
            "posture":"unsupported", "axis":super::vocabulary::axis(value.axis()), "observed":value.observed(),
            "supported":supported(value), "range":range(value.scope().byte_range())}),
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Unknown(
            value,
        )) => json!({
            "posture":"unknown", "reason":super::vocabulary::unknown(value.cause())}),
        PhysicalIntegrityObservationOutcome::Rejected(
            PhysicalIntegrityRejection::Indeterminate(value),
        ) => json!({
            "posture":"indeterminate", "reason":super::vocabulary::indeterminate(value.cause()),
            "observed_range":value.observed_range().map(range)}),
    }
}
fn range(value: PhysicalByteRange) -> Value {
    json!({"offset":value.offset(),"length":value.length()})
}

fn supported(value: UnsupportedPhysicalIntegrityVersion) -> String {
    if value.axis() == PhysicalIntegrityVersionAxis::CheckpointRecordSchema {
        return format!(
            "1|2|{}",
            worth_store_physical_format::CHECKPOINT_CERTIFIED_SCHEMA
        );
    }
    if value.axis() == PhysicalIntegrityVersionAxis::EnvelopeSchema {
        use worth_store_physical_format::DurableFrameKind;
        let kind = match value.scope().artifact_family() {
            PhysicalIntegrityArtifactFamily::RootManifest => Some(DurableFrameKind::RootManifest),
            PhysicalIntegrityArtifactFamily::RootRoutingBlock => {
                Some(DurableFrameKind::RootRoutingBlock)
            }
            _ => None,
        };
        if let Some(kind) = kind {
            return (0..=u8::MAX)
                .filter(|schema| kind.supports_envelope_schema(*schema))
                .map(|schema| schema.to_string())
                .collect::<Vec<_>>()
                .join("|");
        }
    }
    let version = value.scope().format_version();
    match value.axis() {
        PhysicalIntegrityVersionAxis::EnvelopeSchema => version
            .envelope_schema()
            .unwrap_or(version.format_version()),
        _ => version.format_version(),
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };
    use worth_store_physical_format::PhysicalRecordFormatDeclaration;

    #[test]
    fn unsupported_maintenance_schemas_report_complete_supported_sets() {
        let store = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([9; 16]).unwrap(),
        )
        .published_identity();
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let range = PhysicalByteRange::new(0, 360).unwrap();
        let root = PhysicalArtifactScope::root_manifest(store, format, 1, range).unwrap();
        let selector = PhysicalArtifactScope::current_root_selector(store, format, range);
        let checkpoint = PhysicalArtifactScope::checkpoint_stream_header(
            CheckpointStreamHeaderScopeIdentity::staged(store),
            range,
        );
        for (scope, axis, observed, supported) in [
            (
                root,
                PhysicalIntegrityVersionAxis::EnvelopeSchema,
                255,
                "2|3|4|5|6|7|9|10|11",
            ),
            (
                checkpoint,
                PhysicalIntegrityVersionAxis::CheckpointRecordSchema,
                255,
                "1|2|3",
            ),
            (
                selector,
                PhysicalIntegrityVersionAxis::EnvelopeSchema,
                3,
                "2",
            ),
            (root, PhysicalIntegrityVersionAxis::PhysicalFormat, 3, "2"),
        ] {
            let outcome = project(PhysicalIntegrityObservationOutcome::Rejected(
                PhysicalIntegrityRejection::Unsupported(UnsupportedPhysicalIntegrityVersion::new(
                    scope, axis, observed,
                )),
            ));
            assert_eq!(outcome["posture"], "unsupported");
            assert_eq!(outcome["observed"], observed);
            assert_eq!(outcome["supported"], supported);
            assert_eq!(outcome["range"], json!({"offset":0,"length":360}));
        }
    }
}
