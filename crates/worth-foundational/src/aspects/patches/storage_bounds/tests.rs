use super::*;
use crate::aspects::{CanonicalAspectStateMap, PortableRecordAspectPatch};

#[test]
fn owned_patch_storage_bounds_cover_actual_backings_and_checked_overflow() {
    assert_eq!(
        PortableRecordAspectPatch::new([]).readmission_storage_bounds(),
        Some(AspectPatchStorageBounds::default())
    );
    assert_eq!(
        CanonicalAspectStateMap::construction_storage_bytes(0),
        Some(0)
    );
    for count in [1, 2, 3, 4, 7, 8, 9, 65] {
        let bounds =
            AuthoritativeRecordAspectPatch::construction_storage_bounds(count, 0, 0).unwrap();
        let actual = Layout::array::<ContractValidatedAspectValue>(count)
            .unwrap()
            .size();
        assert_eq!(bounds.retained_bytes, actual);
        assert!(
            bounds.transient_bytes >= actual,
            "Vec/Box overlap is funded"
        );
        let field =
            FieldLevelAspectPatch::construction_storage_bounds(count, count, count).unwrap();
        let pairs = Layout::array::<(FieldKey, AspectValue)>(count)
            .unwrap()
            .size();
        let keys = Layout::array::<FieldKey>(count).unwrap().size();
        let paths = Layout::array::<CanonicalFieldPath>(count).unwrap().size();
        assert_eq!(field.retained_bytes, pairs + keys + paths + keys);
        assert!(field.transient_bytes >= pairs + keys + paths + keys);
    }
    assert!(
        AuthoritativeRecordAspectPatch::construction_storage_bounds(usize::MAX, 0, 0).is_none()
    );
    assert!(FieldLevelAspectPatch::construction_storage_bounds(0, usize::MAX, 0).is_none());
    assert!(FieldLevelAspectPatch::construction_storage_bounds(0, 0, usize::MAX).is_none());
    assert!(CanonicalAspectStateMap::construction_storage_bytes(usize::MAX).is_none());
}
