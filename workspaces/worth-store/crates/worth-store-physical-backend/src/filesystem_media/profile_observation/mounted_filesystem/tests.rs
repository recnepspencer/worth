use super::*;

fn observation(fsid: u64, read_only: bool) -> FilesystemObservation {
    FilesystemObservation {
        device: 17,
        fsid,
        granularity: NonZeroU64::new(4096).unwrap(),
        read_only,
    }
}

fn material(read_only: bool) -> MountMaterial {
    MountMaterial {
        filesystem_type: "apfs".into(),
        granularity: NonZeroU64::new(4096).unwrap(),
        location: FilesystemLocation::Local,
        removable: false,
        read_only,
    }
}

#[test]
fn same_device_distinct_filesystems_do_not_conflate_readonly_and_writable_mounts() {
    for read_only in [false, true] {
        let target = observation(if read_only { 35 } else { 33 }, read_only);
        let selected = select_material([
            matching_material(target, observation(35, true), material(true)),
            matching_material(target, observation(33, false), material(false)),
        ])
        .unwrap();
        assert_eq!(selected.read_only, read_only);
    }
}

#[test]
fn every_matching_alias_must_agree_with_target_and_its_own_sysinfo_posture() {
    let target = observation(33, false);
    let mut wrong_granularity = target;
    wrong_granularity.granularity = NonZeroU64::new(8192).unwrap();
    for bad in [
        matching_material(target, observation(33, true), material(true)),
        matching_material(target, target, material(true)),
        matching_material(target, wrong_granularity, material(false)),
    ] {
        assert_eq!(
            select_material([Ok(Some(material(false))), bad]),
            Err(ErrorKind::InvalidData)
        );
    }
}

#[test]
fn matching_aliases_preserve_all_material_conflicts_and_accept_true_aliases() {
    let mut remote = material(false);
    remote.location = FilesystemLocation::Remote;
    let mut removable = material(false);
    removable.removable = true;
    let mut filesystem = material(false);
    filesystem.filesystem_type = "other".into();
    for conflict in [remote, removable, filesystem] {
        assert_eq!(
            select_material([Ok(Some(material(false))), Ok(Some(conflict))]),
            Err(ErrorKind::InvalidData)
        );
    }
    assert_eq!(
        select_material([Ok(Some(material(false))), Ok(Some(material(false)))]),
        Ok(material(false))
    );
}

#[test]
fn absent_and_unobservable_candidates_refuse_even_after_a_valid_candidate() {
    assert_eq!(select_material([Ok(None)]), Err(ErrorKind::NotFound));
    assert_eq!(
        select_material([Ok(Some(material(false))), Err(ErrorKind::PermissionDenied)]),
        Err(ErrorKind::PermissionDenied)
    );
}

#[test]
fn matching_readonly_and_writable_material_reaches_real_profile_policy() {
    use crate::filesystem_media::{MediaCounterSnapshot, MediaQualificationDenial};
    for read_only in [false, true] {
        let observation = observation(33, read_only);
        let selected = select_material([matching_material(
            observation,
            observation,
            material(read_only),
        )])
        .unwrap();
        let observed = profile([1; 32], [2; 32], selected);
        let denial = super::super::deny_profile(&observed, MediaCounterSnapshot::default());
        if read_only {
            assert!(matches!(
                denial,
                Some(MediaQualificationDenial::ReadOnlyFilesystem { .. })
            ));
        } else {
            assert!(denial.is_none());
        }
    }
}
