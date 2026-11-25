//! Shared phase gate bounding Home readmission metadata scans to one active
//! pre-service readmission phase and its sealed owner-admitted metadata kinds.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port::identity::data::primitives::KindId;
use worth_relational_mutation::internal_port::runtime::state::subsystems::schema_contract_runtime::HomeReadmissionMetadataKinds;

#[derive(Debug)]
pub struct HomeReadmissionMetadataPhase {
    runtime_instance_id: u64,
    recovered_latest_commit: Option<CommitId>,
    allowed_kinds: Arc<HomeReadmissionMetadataKinds>,
    active: AtomicBool,
    issued: AtomicBool,
}

impl HomeReadmissionMetadataPhase {
    pub fn new(
        runtime_instance_id: u64,
        recovered_latest_commit: Option<CommitId>,
        allowed_kinds: Arc<HomeReadmissionMetadataKinds>,
    ) -> Self {
        Self {
            runtime_instance_id,
            recovered_latest_commit,
            allowed_kinds,
            active: AtomicBool::new(true),
            issued: AtomicBool::new(false),
        }
    }

    pub fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn recovered_latest_commit(&self) -> Option<CommitId> {
        self.recovered_latest_commit
    }

    pub fn allows_entity_kind(&self, kind_id: KindId) -> bool {
        self.allowed_kinds.contains_entity_kind(kind_id)
    }

    pub fn allows_relation_kind(&self, kind_id: KindId) -> bool {
        self.allowed_kinds.contains_relation_kind(kind_id)
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Mark the single authority issued; false when one was already issued.
    pub fn issue_once(&self) -> bool {
        self.issued
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub fn close(&self) {
        self.active.store(false, Ordering::Release);
    }
}
