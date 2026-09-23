#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalWorkCapacity {
    commands: usize,
    dispatch_permits: usize,
    scope_members_per_work: usize,
    total_scope_members: usize,
    semantic_bytes_per_work: usize,
    total_semantic_bytes: usize,
    terminal_evidence: usize,
}

impl PhysicalWorkCapacity {
    pub fn new(
        commands: usize,
        scope_members_per_work: usize,
        total_scope_members: usize,
        semantic_bytes_per_work: usize,
        total_semantic_bytes: usize,
    ) -> Option<Self> {
        if commands == 0
            || scope_members_per_work == 0
            || total_scope_members < scope_members_per_work
            || semantic_bytes_per_work == 0
            || total_semantic_bytes < semantic_bytes_per_work
        {
            return None;
        }
        Some(Self {
            commands,
            dispatch_permits: commands,
            scope_members_per_work,
            total_scope_members,
            semantic_bytes_per_work,
            total_semantic_bytes,
            terminal_evidence: commands,
        })
    }

    pub fn with_terminal_evidence_capacity(mut self, capacity: usize) -> Option<Self> {
        if capacity == 0 {
            return None;
        }
        self.terminal_evidence = capacity;
        Some(self)
    }

    pub const fn commands(self) -> usize {
        self.commands
    }

    /// Worker permits. Ready slots stay on `commands`. The default keeps them equal.
    pub fn with_dispatch_permits(mut self, permits: usize) -> Option<Self> {
        if permits == 0 {
            return None;
        }
        self.dispatch_permits = permits;
        Some(self)
    }

    pub const fn dispatch_permits(self) -> usize {
        self.dispatch_permits
    }
    pub const fn scope_members_per_work(self) -> usize {
        self.scope_members_per_work
    }
    pub const fn total_scope_members(self) -> usize {
        self.total_scope_members
    }
    pub const fn semantic_bytes_per_work(self) -> usize {
        self.semantic_bytes_per_work
    }
    pub const fn total_semantic_bytes(self) -> usize {
        self.total_semantic_bytes
    }
    pub const fn terminal_evidence(self) -> usize {
        self.terminal_evidence
    }
}

impl Default for PhysicalWorkCapacity {
    fn default() -> Self {
        Self {
            commands: 1_024,
            dispatch_permits: 1_024,
            scope_members_per_work: 256,
            total_scope_members: 32_768,
            semantic_bytes_per_work: 1024 * 1024,
            total_semantic_bytes: 64 * 1024 * 1024,
            terminal_evidence: 4_096,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PhysicalWorkCapacity;

    #[test]
    fn totals_cannot_be_smaller_than_per_work_limits() {
        assert!(PhysicalWorkCapacity::new(1, 2, 1, 1, 1).is_none());
        assert!(PhysicalWorkCapacity::new(1, 1, 1, 2, 1).is_none());
        assert!(PhysicalWorkCapacity::new(1, 1, 1, 1, 1)
            .unwrap()
            .with_terminal_evidence_capacity(0)
            .is_none());
    }

    #[test]
    fn dispatch_permits_default_to_ready_slots_and_reject_zero() {
        let capacity = PhysicalWorkCapacity::new(8, 1, 8, 1, 1).unwrap();
        assert_eq!(capacity.dispatch_permits(), 8);
        assert!(capacity.with_dispatch_permits(0).is_none());
        assert_eq!(
            capacity
                .with_dispatch_permits(4)
                .unwrap()
                .dispatch_permits(),
            4
        );
    }
}
