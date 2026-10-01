use super::*;

impl<'a> AdmittedRootStepMemberView<'a> {
    pub const fn lsn_range(self) -> WalLsnRange {
        self.lsn_range
    }
    pub const fn operation(self) -> [u8; 32] {
        self.operation
    }
    pub const fn group(self) -> PhysicalRedoGroupBinding {
        self.group
    }
    pub const fn fate(self) -> RecoveryOperationFate {
        self.fate
    }
    pub const fn canonical_redo_sha256(self) -> [u8; 32] {
        self.canonical_redo_sha256
    }
    pub const fn materialization(self) -> &'a PersistedPhysicalRecoveryProjection {
        self.materialization
    }
}

impl ImmutablePhysicalRedoPlan {
    /// The digest is authority only when it belongs to this exact unique
    /// semantics-admitted member, not a caller-described WAL byte stream.
    pub(crate) fn admits_exact_member_redo_digest(
        &self,
        projection: &PhysicalRedoProjection,
        digest: [u8; 32],
    ) -> bool {
        let mut members = self
            .projections
            .iter()
            .filter(|member| member.operation() == projection.operation());
        members.next().is_some_and(|member| {
            std::ptr::eq(member, projection)
                && member.group() == projection.group()
                && member.semantics_admitted_redo_sha256() == Some(digest)
        }) && members.next().is_none()
    }

    /// Conservative peak charge for retained descriptors and historical-skip planning.
    pub const fn supersession_scratch_bytes(&self) -> u64 {
        self.scratch_bytes
    }

    pub fn rewrites(&self) -> &[worth_store_physical_format::PhysicalRewriteRedo] {
        &self.rewrites
    }

    pub fn rewrite_admissions(&self) -> &[PhysicalRewriteAdmission] {
        &self.rewrite_admissions
    }

    pub fn install_rewrite_materialization(&mut self, projection: PhysicalRedoProjection) {
        let mut projections = self.projections.to_vec();
        projections.push(projection);
        self.projections = projections.into_boxed_slice();
    }
}

impl PhysicalRewriteAdmission {
    pub const fn operation(self) -> [u8; 32] {
        self.operation
    }

    pub const fn group(self) -> PhysicalRedoGroupBinding {
        self.group
    }

    pub const fn fate(self) -> RecoveryOperationFate {
        self.fate
    }

    pub const fn redo(self) -> worth_store_physical_format::PhysicalRewriteRedo {
        self.redo
    }
}

impl PhysicalRedoProjection {
    pub fn from_rewrite_materialization(
        operation: [u8; 32],
        group: PhysicalRedoGroupBinding,
        fate: RecoveryOperationFate,
        materialization: PersistedPhysicalRecoveryProjection,
    ) -> Self {
        Self {
            operation,
            group,
            fate,
            materialization,
            canonical_redo_sha256: None,
        }
    }
}

impl PhysicalRedoMemberInput {
    pub fn new(
        lsn_range: WalLsnRange,
        operation: [u8; 32],
        fate: RecoveryOperationFate,
        canonical_redo: &[u8],
    ) -> Self {
        let group = PhysicalRedoGroupBinding::new(operation, operation, 1, 1, operation)
            .expect("a singleton synthetic redo group is valid");
        Self::new_grouped(lsn_range, operation, group, fate, canonical_redo)
    }

    pub fn new_grouped(
        lsn_range: WalLsnRange,
        operation: [u8; 32],
        group: PhysicalRedoGroupBinding,
        fate: RecoveryOperationFate,
        canonical_redo: &[u8],
    ) -> Self {
        Self {
            lsn_range,
            operation,
            group,
            fate,
            canonical_redo: canonical_redo.into(),
        }
    }

    pub const fn lsn_range(&self) -> WalLsnRange {
        self.lsn_range
    }
    pub const fn operation(&self) -> [u8; 32] {
        self.operation
    }
    pub const fn group(&self) -> PhysicalRedoGroupBinding {
        self.group
    }
    pub const fn fate(&self) -> RecoveryOperationFate {
        self.fate
    }
    pub fn canonical_redo(&self) -> &[u8] {
        &self.canonical_redo
    }
}

impl PhysicalRedoGroupBinding {
    pub const fn new(
        group_identity: [u8; 32],
        member_identity: [u8; 32],
        member_ordinal: u32,
        member_count: u32,
        membership_digest: [u8; 32],
    ) -> Option<Self> {
        if member_ordinal == 0 || member_count == 0 || member_ordinal > member_count {
            return None;
        }
        Some(Self {
            group_identity,
            member_identity,
            member_ordinal,
            member_count,
            membership_digest,
        })
    }
    pub const fn group_identity(self) -> [u8; 32] {
        self.group_identity
    }
    pub const fn member_identity(self) -> [u8; 32] {
        self.member_identity
    }
    pub const fn member_ordinal(self) -> u32 {
        self.member_ordinal
    }
    pub const fn member_count(self) -> u32 {
        self.member_count
    }
    pub const fn membership_digest(self) -> [u8; 32] {
        self.membership_digest
    }
}

impl ImmutablePhysicalRedoPlan {
    pub fn decisions(&self) -> &[PhysicalRedoDecision] {
        &self.decisions
    }
    pub fn resolved_decisions(
        &self,
    ) -> impl ExactSizeIterator<Item = PhysicalRedoDecisionView<'_>> {
        self.decisions.iter().map(|decision| {
            let record = &self.records[decision.record_index as usize];
            let target = &record.targets()[decision.target_index as usize];
            PhysicalRedoDecisionView {
                decision,
                record,
                target,
            }
        })
    }
    pub fn projections(&self) -> &[PhysicalRedoProjection] {
        &self.projections
    }
    /// The already-admitted canonical record for a single-record blob semantic
    /// member. Multiple target decisions may name the same record.
    pub fn blob_semantic_record_bytes(&self, operation: [u8; 32]) -> Option<&[u8]> {
        let mut records = self
            .decisions
            .iter()
            .filter(|decision| decision.operation == operation)
            .map(|decision| decision.record_index);
        let first = records.next()?;
        if records.any(|index| index != first) {
            return None;
        }
        self.records
            .get(first as usize)
            .map(PhysicalRedoRecord::bytes)
    }
    pub const fn recovery_root_allocation_bytes(&self) -> u64 {
        self.recovery_root_allocation_bytes
    }
    pub const fn counters(&self) -> PhysicalRedoPlanCounters {
        self.counters
    }

    pub fn operation_group_is_fully_materialized(&self, operation: [u8; 32]) -> bool {
        let Some(group) = self
            .projections
            .iter()
            .find(|projection| projection.operation == operation)
            .map(|projection| projection.group)
        else {
            return false;
        };
        let members = self
            .projections
            .iter()
            .filter(|projection| projection.group.group_identity() == group.group_identity())
            .collect::<Vec<_>>();
        members.len() == group.member_count() as usize
            && members.iter().all(|member| {
                let decisions = self
                    .decisions
                    .iter()
                    .filter(|decision| decision.operation == member.operation)
                    .collect::<Vec<_>>();
                !decisions.is_empty()
                    && decisions.iter().all(|decision| {
                        matches!(
                            decision.kind,
                            PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
                                | PhysicalRedoDecisionKind::SkipOperationAlreadyMaterialized
                        )
                    })
            })
    }
}

impl PhysicalRedoProjection {
    pub const fn semantics_admitted_redo_sha256(&self) -> Option<[u8; 32]> {
        self.canonical_redo_sha256
    }
    pub const fn operation(&self) -> [u8; 32] {
        self.operation
    }
    pub const fn group(&self) -> PhysicalRedoGroupBinding {
        self.group
    }
    pub const fn fate(&self) -> RecoveryOperationFate {
        self.fate
    }
    pub const fn materialization(&self) -> &PersistedPhysicalRecoveryProjection {
        &self.materialization
    }
}

impl<'plan> PhysicalRedoDecisionView<'plan> {
    pub const fn kind(self) -> PhysicalRedoDecisionKind {
        self.decision.kind
    }
    pub const fn operation(self) -> [u8; 32] {
        self.decision.operation
    }
    pub const fn prior(self) -> PhysicalRedoDecisionPrior {
        self.decision.prior
    }
    pub const fn record_index(self) -> u64 {
        self.decision.record_index
    }
    pub const fn target_index(self) -> u64 {
        self.decision.target_index
    }
    pub const fn record(self) -> &'plan PhysicalRedoRecord {
        self.record
    }
    pub const fn target(self) -> &'plan PhysicalRedoTarget {
        self.target
    }
}

impl PhysicalRedoDecision {
    pub const fn kind(&self) -> PhysicalRedoDecisionKind {
        self.kind
    }
    pub const fn operation(&self) -> [u8; 32] {
        self.operation
    }
    pub const fn prior(&self) -> PhysicalRedoDecisionPrior {
        self.prior
    }
    pub const fn record_index(&self) -> u64 {
        self.record_index
    }
    pub const fn target_index(&self) -> u64 {
        self.target_index
    }
}

impl PhysicalRedoPlanCounters {
    pub const fn records(self) -> u64 {
        self.records
    }
    pub const fn targets(self) -> u64 {
        self.targets
    }
    pub const fn apply(self) -> u64 {
        self.apply
    }
    pub const fn skip_page_lsn(self) -> u64 {
        self.skip_page_lsn
    }
    pub const fn skip_historical_drop(self) -> u64 {
        self.skip_historical_drop
    }
    pub const fn skip_operation(self) -> u64 {
        self.skip_operation
    }
}
