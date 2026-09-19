use worth_ui::facade::app::{
    UiMountedInspectionReceipt, UiMountedInspectionRequest, WorthUiNativeApplicationShell,
};
use worth_ui::facade::inspection::{
    UiPixelsRequired, UiVisualSnapshotReceipt, UiVisualSnapshotRelation,
    UiVisualSnapshotRelationDenial,
};

use super::PlatformPulseVisualExecutionDenial;

pub(super) fn snapshot_matches_current_mounted_frame(
    snapshot: &UiVisualSnapshotReceipt<UiPixelsRequired>,
    shell: &WorthUiNativeApplicationShell,
) -> Result<bool, PlatformPulseVisualExecutionDenial> {
    let relation = match snapshot.relation() {
        Ok(relation) => relation,
        // A retained snapshot whose frame retention already expired or forgot
        // is a superseded predecessor: not current, and refreshable through
        // the same refresh path whose retirement accepts those relations.
        Err(UiVisualSnapshotRelationDenial::ExpiredFrame)
        | Err(UiVisualSnapshotRelationDenial::UnknownFrame) => return Ok(false),
        Err(denial) => return Err(PlatformPulseVisualExecutionDenial::SnapshotRelation(denial)),
    };
    let current = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
        UiMountedInspectionReceipt::Available(frame) => Some(frame.frame().diagnostic_value()),
        UiMountedInspectionReceipt::Omitted(
            worth_ui::facade::app::UiMountedInspectionOmission::FrameTransitionInFlight,
        ) => None,
        UiMountedInspectionReceipt::Omitted(omission) => {
            return Err(
                PlatformPulseVisualExecutionDenial::ComparisonMountedFrameUnavailable(omission),
            )
        }
    };
    Ok(snapshot_affinity_is_current(
        relation,
        snapshot.affinity().frame(),
        current,
    ))
}

fn snapshot_affinity_is_current(
    relation: UiVisualSnapshotRelation,
    snapshot_frame: u64,
    current_frame: Option<u64>,
) -> bool {
    relation == UiVisualSnapshotRelation::Current && current_frame == Some(snapshot_frame)
}

#[cfg(test)]
mod tests {
    use super::{snapshot_affinity_is_current, UiVisualSnapshotRelation};

    #[test]
    fn completed_capture_is_revalidated_against_the_current_mounted_frame() {
        assert!(!snapshot_affinity_is_current(
            UiVisualSnapshotRelation::Current,
            41,
            Some(42),
        ));
        assert!(!snapshot_affinity_is_current(
            UiVisualSnapshotRelation::Current,
            42,
            None,
        ));
        assert!(snapshot_affinity_is_current(
            UiVisualSnapshotRelation::Current,
            42,
            Some(42),
        ));
    }
}
