//! A genuine process-killed V3 released drop, retained for C8/Store rejoin
//! tests that must share one pending descriptor and its physical controls.

pub use {std::cell::Cell, std::fs, std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};

pub use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReadOpenFailure, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordReadDenial};
pub use {worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

pub use {std::num::NonZeroU64, crate::production_entry::blob_scope_admission::admitted_blob_scope, crate::production_entry::certified_release_serving};








pub fn selected_session_declared_extent(
    root: &Path,
) -> worth_store_physical_format::ExtentArenaRange {
    crate::pending_wal_world::selected_media::selected_session_declared_extent(root)
}

pub const ROLE_ENV: &str = "WORTH_C11_PENDING_V3_CHILD_ROLE";
pub const MARKER_ENV: &str = "WORTH_C11_PENDING_V3_CHILD_MARKER";
pub const ROOT_ENV: &str = "WORTH_C11_PENDING_V3_CHILD_ROOT";
pub const WAL_SEGMENT_BYTES_ENV: &str = "WORTH_C11_PENDING_V3_WAL_SEGMENT_BYTES";
pub const FIRST_WORLD_ENV: &str = "WORTH_C11_PENDING_V3_FIRST_WORLD";
pub const CHILD_TEST: &str = "pending_wal_world::pending_wal_child";
pub const CHUNK: usize = 64 << 10;

pub use {crate::pending_wal_world::descriptor_park::park_at_descriptor_wal};
pub use {};

pub struct PendingWalWorld {
    pub root: PathBuf,
    pub first: first::World,
    pub objects: Vec<[u8; 16]>,
    pub _marker: tempfile::TempDir,
    pub next_descriptor: Cell<u64>,
}

impl PendingWalWorld {
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// A recovery request with the limits this world's WAL and batches need.
    pub fn recovery_request(
        &self,
    ) -> worth_store_recovery_runtime::PhysicalRecoveryOpenRequest {
        self.first.recovery_request(&self.root)
    }

    /// Requires every object the first child published and did not release
    /// to read back byte-exact, and the released one to be gone.
    pub fn assert_objects_read_back(
        &self,
        serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    ) {
        let scope = admitted_blob_scope("c11.recovery.pending-v3.scope");
        let limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
        let blobs = serving.blobs().unwrap();
        let expected = self.first.object_bytes();
        assert_eq!(self.objects.len(), expected.len());
        for (index, (object, expected)) in self.objects.iter().zip(expected).enumerate() {
            let resolved = blobs.resolve_publication(*object, 1, &scope, limits);
            if index == 0 && self.first.releases_first_object() {
                match resolved {
                    Err(worth_store::physical_runtime::BlobReadOpenFailure::PublicationNotFound) => {}
                    Err(worth_store::physical_runtime::BlobReadOpenFailure::Read(worth_store::physical_runtime::BlobReadFailure::RecordRead(error)))
                        if error.denial() == worth_store::physical_runtime::RecordReadDenial::RecordNotFound => {}
                    other => panic!("released first object remained visible: {other:?}"),
                }
                continue;
            }
            let mut read = blobs
                .read(resolved.unwrap(), &scope, 0, expected.len() as u64, limits)
                .unwrap();
            let mut observed = vec![0_u8; expected.len()];
            let mut used = 0;
            while used < observed.len() {
                let count = read.read_next(&mut observed[used..]).unwrap();
                assert!(count > 0, "object {index} ended at byte {used}");
                used += count;
            }
            assert!(observed == expected, "object {index} read back changed");
        }
    }

    pub fn kill_second_after_certified_retirement(&self) {
        self.kill_next_descriptor("second", true);
    }

    pub fn kill_distinct_release_before_checkpoint(&self) {
        self.kill_next_descriptor("distinct", false);
    }

    /// Parks the next one-record batch of the first object. No retirement and
    /// no checkpoint: the batch extends whatever the recovered world holds.
    pub fn kill_successor_of_first_object(&self) {
        self.kill_next_descriptor("successor-first", false);
    }

    /// Parks the next batch of the first object at the whole manifest
    /// capacity. A remainder that fits leaves whole, which makes the batch
    /// terminal.
    pub fn kill_full_batch_successor_of_first_object(&self) {
        self.kill_next_descriptor("successor-first-full", false);
    }

    /// Parks the next one-record batch of the one distinct object.
    pub fn kill_successor_of_distinct_object(&self) {
        self.kill_next_descriptor("successor-distinct", false);
    }

    pub fn kill_next_descriptor(&self, role: &str, checkpoint_changes: bool) {
        let ordinal = self.next_descriptor.get();
        self.next_descriptor
            .set(ordinal.checked_add(1).expect("bounded fixture sequence"));
        let marker = self
            ._marker
            .path()
            .join(format!("descriptor-{ordinal}-ready"));
        let checkpoint_before = std::fs::read(self.root.join("families/checkpoint.current")).unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
            .env(ROLE_ENV, role)
            .env(FIRST_WORLD_ENV, self.first.role())
            .env(ROOT_ENV, &self.root)
            .env(MARKER_ENV, &marker)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(240);
        while !marker.is_file() {
            if child.try_wait().unwrap().is_some() || std::time::Instant::now() >= deadline {
                let _ = child.kill();
                let output = child.wait_with_output().unwrap();
                panic!(
                    "second pending V3 descriptor seam not reached: {} {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        child.kill().unwrap();
        child.wait().unwrap();
        assert_eq!(std::fs::read(&marker).unwrap(), b"second-descriptor-wal");
        assert_eq!(
            std::fs::read(self.root.join("families/checkpoint.current")).unwrap() != checkpoint_before,
            checkpoint_changes,
            "only certified retirement may advance the checkpoint before the next descriptor"
        );
    }
}

impl Drop for PendingWalWorld {
    fn drop(&mut self) {
        let resolved = self
            .root
            .canonicalize()
            .expect("pending world root remains");
        assert!(resolved.starts_with(std::env::temp_dir().canonicalize().unwrap()));
        assert!(resolved
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("worth-store-c11-pending-v3-"));
        std::fs::remove_dir_all(resolved).expect("remove exact child-owned pending world");
    }
}