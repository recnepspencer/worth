#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecoveryCoordinationCapacity {
    commands: usize,
    semantic_bytes: usize,
    cleanup_candidates: usize,
    cleanup_bytes: u64,
    recovery_allocation_bytes: Option<u64>,
}

impl PhysicalRecoveryCoordinationCapacity {
    pub fn admit(
        commands: u64,
        semantic_bytes: u64,
        cleanup_candidates: u64,
        cleanup_bytes: u64,
    ) -> Option<Self> {
        let commands = usize::try_from(commands).ok()?;
        let semantic_bytes = usize::try_from(semantic_bytes).ok()?;
        let cleanup_candidates = usize::try_from(cleanup_candidates).ok()?;
        (commands != 0 && semantic_bytes != 0 && cleanup_candidates != 0 && cleanup_bytes != 0)
            .then_some(Self {
                commands,
                semantic_bytes,
                cleanup_candidates,
                cleanup_bytes,
                recovery_allocation_bytes: None,
            })
    }

    /// Admits the recovery-resident closure limit for a selected V2 head
    /// roster. The live coordination owner binds it to concrete Store media.
    pub fn with_recovery_allocation_bytes(mut self, bytes: u64) -> Option<Self> {
        if bytes == 0 {
            return None;
        }
        self.recovery_allocation_bytes = Some(bytes);
        Some(self)
    }

    pub(super) const fn recovery_allocation_bytes(self) -> Option<u64> {
        self.recovery_allocation_bytes
    }

    pub(super) fn work_capacity(self) -> crate::physical_runtime::PhysicalWorkCapacity {
        crate::physical_runtime::PhysicalWorkCapacity::new(
            self.commands,
            1,
            self.commands,
            self.semantic_bytes,
            self.semantic_bytes,
        )
        .expect("admission established nonzero recovery coordination capacity")
    }

    pub(in crate::physical_runtime) const fn cleanup_candidates(self) -> usize {
        self.cleanup_candidates
    }

    pub(in crate::physical_runtime) const fn cleanup_bytes(self) -> u64 {
        self.cleanup_bytes
    }
}
