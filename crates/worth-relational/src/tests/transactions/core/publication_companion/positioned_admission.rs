use super::*;

#[test]
fn exact_snapshot_position_is_admitted_before_native_binding_copies() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "position-admission-anchor");
    let handle = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
    let expected = runtime
        .read_truth()
        .positioned_snapshot(&handle)
        .expect("the issued snapshot has a canonical position");

    let mut total_work = 0_u64;
    let mut total_bytes = 0_u64;
    let admitted = runtime
        .read_truth()
        .positioned_snapshot_admitted(&handle, |work, bytes| {
            total_work += work;
            total_bytes += bytes;
            Ok::<_, ()>(())
        })
        .expect("the owner admits the actual issued snapshot");
    assert_eq!(admitted, expected);
    assert!(total_work > 3);
    assert!(total_bytes > 0);

    let mut spent = 0_u64;
    let one_work_short = runtime
        .read_truth()
        .positioned_snapshot_admitted(&handle, |work, _| {
            spent += work;
            if spent > total_work - 1 {
                Err("work")
            } else {
                Ok(())
            }
        });
    assert!(matches!(
        one_work_short,
        Err(crate::runtime::RelationalSnapshotPositionAdmissionStop::Admission("work"))
    ));

    let mut spent_bytes = 0_u64;
    let one_byte_short = runtime
        .read_truth()
        .positioned_snapshot_admitted(&handle, |_, bytes| {
            spent_bytes += bytes;
            if spent_bytes > total_bytes - 1 {
                Err("memory")
            } else {
                Ok(())
            }
        });
    assert!(matches!(
        one_byte_short,
        Err(crate::runtime::RelationalSnapshotPositionAdmissionStop::Admission("memory"))
    ));
}
