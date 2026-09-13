use super::{
    decode_root_record, physical_access::read_leaf, BaselineBTreeExactCounterWitness,
    BaselineBTreeExecutionDenial, BaselineBTreeReplayAdmission,
    BaselineBTreeReplayRecoveryExecution,
};
use sha2::{Digest, Sha256};
use worth_store_contracts::AcceptedHandoffReadiness;
use worth_store_physical_format::{
    access::page::PageAccess, InMemoryPhysicalFormatReplayArtifact, PhysicalReference,
    PhysicalStoreIdentity,
};
use worth_store_recovery_physics::PhysicalSourceSelection;

use crate::{AdmittedBTreeReplayPhysicalSource, AdmittedBTreeReplaySource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BTreeReplayRuntime;

#[derive(Debug, PartialEq, Eq)]
pub struct BTreeReplayReady {
    source: AdmittedBTreeReplaySource<BaselineBTreeReplayAdmission>,
    current_materialization: crate::CurrentLayoutMaterialization,
}

impl BTreeReplayReady {
    fn issue(
        source: AdmittedBTreeReplaySource<BaselineBTreeReplayAdmission>,
    ) -> Result<Self, BaselineBTreeExecutionDenial> {
        let current_materialization =
            crate::CurrentLayoutMaterialization::from_btree_replay_source(&source)
                .map_err(|_| BaselineBTreeExecutionDenial::WrongSelectedOperation)?;
        Ok(Self {
            source,
            current_materialization,
        })
    }
}

impl BTreeReplayRuntime {
    pub fn admit_physical_source(
        self,
        readiness: AcceptedHandoffReadiness,
        root_reference: PhysicalReference,
        replay_artifact: InMemoryPhysicalFormatReplayArtifact,
        expected_store_identity: PhysicalStoreIdentity,
        durable_source: PhysicalSourceSelection,
    ) -> Result<AdmittedBTreeReplayPhysicalSource, BaselineBTreeExecutionDenial> {
        Ok(AdmittedBTreeReplayPhysicalSource::admit(
            readiness,
            root_reference,
            replay_artifact,
            expected_store_identity,
            durable_source,
        )?)
    }

    pub fn bind_source(
        self,
        admission: BaselineBTreeReplayAdmission,
        source: AdmittedBTreeReplayPhysicalSource,
    ) -> Result<BTreeReplayReady, BaselineBTreeExecutionDenial> {
        BTreeReplayReady::issue(source.bind_intent(admission))
    }

    pub fn execute(
        self,
        ready: BTreeReplayReady,
    ) -> Result<BaselineBTreeReplayRecoveryExecution, BaselineBTreeExecutionDenial> {
        let BTreeReplayReady {
            source,
            current_materialization,
        } = ready;
        let mut facade = source.reopen()?;
        let (root_counters, node) = {
            let mut root_access = facade.page_access();
            let root = root_access.read_record(source.root_reference())?;
            let root_counters = PageAccess::access_counters(root);
            let node = decode_root_record(root.record_view().payload().as_bytes())
                .ok_or(BaselineBTreeExecutionDenial::InvalidRootNode)?;
            (root_counters, node)
        };
        let left = read_leaf(&mut facade, node.left_child())?;
        let right = read_leaf(&mut facade, node.right_child())?;
        let authority_records = left
            .leaf
            .slots()
            .len()
            .saturating_add(right.leaf.slots().len()) as u16;
        let exact_counters = BaselineBTreeExactCounterWitness::from_replay_reads([
            root_counters,
            left.counters,
            right.counters,
        ]);
        let admission = source.intent();
        let recovery_source_digest = selection_digest(source.durable_source());
        Ok(BaselineBTreeReplayRecoveryExecution::new(
            admission.plan_binding().clone(),
            admission.request_identity(),
            source.persisted_layout().clone(),
            source.root_reference(),
            source.persisted_layout().root_manifest_candidates()[0].clone(),
            authority_records,
            authority_records,
            true,
            exact_counters,
            recovery_source_digest,
            current_materialization,
        ))
    }
}

pub const fn btree_replay_runtime() -> BTreeReplayRuntime {
    BTreeReplayRuntime
}

fn selection_digest(selection: &PhysicalSourceSelection) -> String {
    let mut digest = Sha256::new();
    digest.update(format!("{:?}", selection.trace()).as_bytes());
    digest.update(
        selection
            .checkpoint()
            .map_or(0, |checkpoint| {
                checkpoint.checkpoint().source().identity().sequence().get()
            })
            .to_le_bytes(),
    );
    digest.update(selection.wal_tail().frame_count().to_le_bytes());
    digest.update(selection.wal_tail().byte_count().to_le_bytes());
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
