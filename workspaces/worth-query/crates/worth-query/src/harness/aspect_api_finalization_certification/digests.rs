use crate::harness::certification::digest_parts;
use crate::runtime::{WorthQueryAspectTouch, WorthQueryInspection};

pub(super) fn inspection_digest(inspection: &WorthQueryInspection) -> String {
    match inspection {
        WorthQueryInspection::WriteReceipt(inspection) => {
            inspection.inspection_digest().to_string()
        }
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            inspection.inspection_digest().to_string()
        }
        other => panic!("expected mutation inspection, got {other:?}"),
    }
}

pub(super) fn touched_aspect_digest(touches: &[WorthQueryAspectTouch]) -> String {
    digest_parts(
        &touches
            .iter()
            .map(|touch| format!("aspect:{}", touch.admitted_touch_digest_part()))
            .collect::<Vec<_>>(),
    )
}
