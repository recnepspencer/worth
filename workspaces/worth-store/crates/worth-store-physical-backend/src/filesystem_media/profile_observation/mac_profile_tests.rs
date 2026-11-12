use super::*;
use crate::filesystem_media::{
    FilesystemAccessPosture, FilesystemMediaOwner, FilesystemQualificationRequest,
    MediaCounterSnapshot, MediaQualificationDenial, MediaQualificationStale,
};
use worth_proof::TransitionOutcome;

#[test]
fn path_and_held_root_profiles_match_the_actual_opened_filesystem() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let request = FilesystemQualificationRequest::production(
        &root,
        FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let TransitionOutcome::Success(media) = FilesystemMediaOwner::qualify(request).into_raw()
    else {
        panic!("actual Mac production qualification must succeed");
    };
    let (owner, profile, _, _, _) = media.into_runtime_parts();
    let from_path = observe_profile_at_path(&root).unwrap();
    let from_handle = observe_profile_at_handle(owner.root_directory_handle()).unwrap();
    assert_eq!(profile, from_path);
    assert_eq!(profile, from_handle);
    let file = cap_std::fs::Dir::open_ambient_dir(&root, cap_std::ambient_authority())
        .unwrap()
        .into_std_file();
    let os = rustix::fs::fstatvfs(&file).unwrap();
    assert_eq!(profile.allocation_granularity().get(), os.f_frsize);
    assert_eq!(
        profile.is_read_only(),
        os.f_flag.contains(rustix::fs::StatVfsMountFlags::RDONLY)
    );
    assert!(!profile.is_read_only(), "normal temp root must be writable");
    owner.close();
}

#[test]
fn actual_readonly_root_distinguishes_profile_policy_from_unavailable_observation() {
    let root = Path::new("/");
    let request = FilesystemQualificationRequest::production(
        root,
        FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    match observe_profile_at_path(root) {
        Ok(profile) => {
            assert!(
                profile.is_read_only(),
                "host readonly fixture must actually be readonly"
            );
            assert!(matches!(
                deny_profile(&profile, MediaCounterSnapshot::default()),
                Some(MediaQualificationDenial::ReadOnlyFilesystem { .. })
            ));
            assert!(matches!(
                FilesystemMediaOwner::qualify(request).into_raw(),
                TransitionOutcome::Denied(MediaQualificationDenial::ReadOnlyFilesystem { .. })
            ));
            eprintln!("actual root candidate reaches ReadOnlyFilesystem policy denial");
        }
        Err(kind) => {
            assert!(matches!(
                kind,
                std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidData
            ));
            assert!(matches!(
                FilesystemMediaOwner::qualify(request).into_raw(),
                TransitionOutcome::Stale(MediaQualificationStale::RootUnavailable { .. })
            ));
            eprintln!("actual root profile unavailable ({kind:?}); host readonly-policy proof UNAVAILABLE");
        }
    }
}
