use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::super::{NativeDesktopCourtroomLease, NativeDesktopLease, NativeDesktopLeaseFailure};
use super::LeaseAttempt;

/// A lock file this test plants and removes again even when an assertion
/// fires first.
struct PlantedLock {
    path: PathBuf,
    file: File,
}

impl PlantedLock {
    fn locked_by_this_process(path: PathBuf) -> Self {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .expect("plant lock file");
        file.try_lock().expect("plant the lock itself");
        Self { path, file }
    }

    fn unlocked(path: PathBuf, content: &str) -> Self {
        std::fs::write(&path, content).expect("plant unlocked lock file");
        let file = File::open(&path).expect("open planted file");
        Self { path, file }
    }
}

impl Drop for PlantedLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = self.file.unlock();
    }
}

/// A pid-keyed lock path this test owns outright: nothing is planted, the
/// test's own attempts create the inode, and the path is removed again even
/// when an assertion fires first.
struct ScratchLockPath(PathBuf);

impl ScratchLockPath {
    fn fresh(purpose: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "worth-ui-native-desktop-{purpose}-{}.lock",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        Self(path)
    }
}

impl Drop for ScratchLockPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn the_lease_is_keyed_by_display_and_survives_display_punctuation() {
    assert_eq!(
        super::lease_file_name(Some(":0")),
        "worth-ui-native-desktop-_0-v1.lock"
    );
    assert_eq!(
        super::lease_file_name(Some("localhost:10.0")),
        "worth-ui-native-desktop-localhost_10_0-v1.lock"
    );
    assert_eq!(
        super::lease_file_name(None),
        "worth-ui-native-desktop-unset-v1.lock"
    );
    assert_eq!(
        super::lease_file_name(Some("")),
        super::lease_file_name(None)
    );
}

/// A lock file whose owner died is an unlocked file: the kernel released it,
/// whatever pid it still names.
#[test]
fn a_lock_left_by_a_dead_owner_is_taken_at_once_and_names_the_new_owner() {
    let _courtroom = NativeDesktopCourtroomLease::acquire();
    let path = super::lease_path();
    let planted = PlantedLock::unlocked(path.clone(), "4294967295");
    let lease = NativeDesktopLease::acquire(Instant::now() + Duration::from_millis(200))
        .expect("an unlocked file is free");
    assert_eq!(super::holder_process_id(&path), Some(std::process::id()));
    drop(planted);
    drop(lease);
    assert!(!path.exists(), "released lease removes its file");
}

/// A lock held through another open file description contends, even inside
/// this process: `flock` is per description, not per process.
#[test]
fn a_lock_held_by_a_live_owner_is_contended_until_the_deadline() {
    let _courtroom = NativeDesktopCourtroomLease::acquire();
    let path = super::lease_path();
    let planted = PlantedLock::locked_by_this_process(path.clone());
    let failure = NativeDesktopLease::acquire(Instant::now() + Duration::from_millis(30))
        .err()
        .expect("live owner blocks acquisition");
    assert!(
        matches!(failure, NativeDesktopLeaseFailure::Deadline),
        "{failure}"
    );
    assert!(path.exists(), "contention leaves the owner's file alone");
    drop(planted);
}

/// A lock on an inode the owner has already unlinked is an orphan, not the
/// lease: the contender must see the file at `path` change under it.
#[test]
fn a_lock_on_an_unlinked_inode_is_not_ownership() {
    let _courtroom = NativeDesktopCourtroomLease::acquire();
    let path = super::lease_path();
    let orphan = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .expect("open the inode a contender would hold");
    std::fs::remove_file(&path).expect("owner unlinks while locked");
    orphan.try_lock().expect("the orphan inode is lockable");
    assert!(matches!(
        super::locked_inode_is_at_path(&orphan, &path),
        Ok(false)
    ));
    let lease = NativeDesktopLease::acquire(Instant::now() + Duration::from_millis(200))
        .expect("a fresh inode at the path is free");
    assert!(matches!(
        super::locked_inode_is_at_path(&orphan, &path),
        Ok(false)
    ));
    drop(lease);
}

/// FALSIFIER for the two-owner defect (gate round 7, B2): several processes
/// start on one unheld path at the same moment and each holds what it wins
/// until it exits. Exactly one may own; the rest must fail typed at their
/// deadline. The old pid-liveness reclaim granted two owners here.
#[test]
fn simultaneous_contenders_on_one_desktop_yield_exactly_one_owner() {
    const CHILD_MARKER: &str = "WORTH_UI_DESKTOP_LEASE_RACE_CHILD";
    const CONTENDERS: usize = 6;
    const OWNED: &str = "lease-race-owned";
    const DEADLINE: &str = "lease-race-deadline";
    if std::env::var_os(CHILD_MARKER).is_some() {
        match NativeDesktopLease::acquire(Instant::now() + Duration::from_millis(300)) {
            Ok(lease) => {
                println!("{OWNED}");
                std::thread::sleep(Duration::from_millis(400));
                drop(lease);
            }
            Err(NativeDesktopLeaseFailure::Deadline) => println!("{DEADLINE}"),
            Err(failure) => panic!("contender failed outside contention: {failure}"),
        }
        return;
    }
    let _courtroom = NativeDesktopCourtroomLease::acquire();
    let path = super::lease_path();
    let _ = std::fs::remove_file(&path);
    let children = (0..CONTENDERS)
        .map(|_| {
            std::process::Command::new(std::env::current_exe().unwrap())
                .env(CHILD_MARKER, "1")
                .args([
                    "product_process::native_desktop_lease::linux::tests::simultaneous_contenders_on_one_desktop_yield_exactly_one_owner",
                    "--exact",
                    "--nocapture",
                ])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("spawn lease contender")
        })
        .collect::<Vec<_>>();
    let mut owned = 0;
    let mut deadline = 0;
    for child in children {
        let output = child.wait_with_output().expect("contender exits");
        assert!(
            output.status.success(),
            "contender failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        owned += stdout.matches(OWNED).count();
        deadline += stdout.matches(DEADLINE).count();
    }
    assert_eq!(owned, 1, "exactly one contender may own the desktop");
    assert_eq!(
        deadline,
        CONTENDERS - 1,
        "every other contender fails typed"
    );
    assert!(!path.exists(), "the owner's release removed the lock file");
}

/// FALSIFIER for the release order (gate round 8, R8-1): the owner must
/// unlink its file *before* the kernel drops its lock. A contender planted
/// between the two steps owns a fresh inode, so the contender that follows
/// the release finds that inode locked and contends. In the opposite order
/// the planted contender locks the old inode, the release then unlinks it,
/// and the following contender owns a second inode: two owners of one
/// desktop through the public path.
#[test]
fn release_unlinks_before_it_unlocks_so_a_contender_between_the_steps_stays_sole_owner() {
    let scratch = ScratchLockPath::fresh("release-order");
    let path = scratch.0.clone();
    let owner = match super::attempt(&path) {
        LeaseAttempt::Owned(file) => file,
        other => panic!("an unheld path is owned: {other:?}"),
    };
    let mut between_steps = None;
    super::release_with_contention_between(owner, &path, || {
        between_steps = Some(super::attempt(&path));
    });
    let between_owner = match between_steps {
        Some(LeaseAttempt::Owned(file)) => file,
        other => panic!("the path is free once unlinked: {other:?}"),
    };
    let after_release = super::attempt(&path);
    assert!(
        matches!(after_release, LeaseAttempt::Contended),
        "the between-steps owner must still hold the inode at the path: {after_release:?}"
    );
    assert!(matches!(
        super::locked_inode_is_at_path(&between_owner, &path),
        Ok(true)
    ));
    super::release(between_owner, &path);
    assert!(
        !path.exists(),
        "the sole owner's release removed the lock file"
    );
}
