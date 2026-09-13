use serde_json::{json, Value};
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
    let version = value.scope().format_version();
    match value.axis() {
        PhysicalIntegrityVersionAxis::EnvelopeSchema => version
            .envelope_schema()
            .unwrap_or(version.format_version()),
        _ => version.format_version(),
    }
    .to_string()
}
