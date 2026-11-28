use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::temporal::AdmittedBridgeTemporalBasis;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTemporalAsyncCertificationBasisSection {
    temporal_basis_identity: Arc<str>,
    truth_basis_kind: Arc<str>,
    truth_branch_identity: Arc<str>,
    truth_snapshot_identity: Arc<str>,
    truth_locator_identity: Arc<str>,
    signal_clock_domain: Arc<str>,
    signal_clock_tick: u64,
    signal_clock_advance_ordinal: u64,
    signal_clock_checkpoint: Arc<str>,
    wake_id: u64,
    wake_ready_ordinal: u64,
    wake_tick: u64,
    truth_owner: Arc<str>,
    signal_owner: Arc<str>,
    bridge_owner: Arc<str>,
    semantic_digest: Arc<str>,
    digest: Arc<str>,
}

impl BridgeTemporalAsyncCertificationBasisSection {
    pub(crate) fn collect(temporal_basis: &AdmittedBridgeTemporalBasis) -> Self {
        let truth_basis = temporal_basis.truth_basis().basis();
        let signal_basis = temporal_basis.signal_basis().basis();
        let wake = temporal_basis.wake_evidence().evidence();
        let truth_basis_kind = Arc::from(format!("{:?}", truth_basis.kind()).to_lowercase());
        let truth_branch_identity =
            Arc::from(truth_basis.branch_identity().canonical_label().to_owned());
        let truth_snapshot_identity =
            Arc::from(truth_basis.snapshot_identity().canonical_label().to_owned());
        let truth_locator_identity = Arc::from(truth_basis.native_truth_locator().to_owned());
        let signal_clock_domain = Arc::from(format!("{:?}", signal_basis.clock_domain()));
        let signal_clock_checkpoint = Arc::from(
            signal_basis
                .last_checkpoint_id()
                .map(|checkpoint| checkpoint.get().to_string())
                .unwrap_or_else(|| "none".to_owned()),
        );
        let semantic_basis = format!(
            "bridge-temporal-async-certification-basis-semantic|temporal={}|truth-kind={}|truth-branch={}|truth-snapshot={}|truth-locator={}|signal-domain={}|signal-tick={}|signal-advance={}|signal-checkpoint={}|wake-id={}|wake-ordinal={}|wake-tick={}",
            temporal_basis.identity().as_str(),
            truth_basis_kind,
            truth_branch_identity,
            truth_snapshot_identity,
            truth_locator_identity,
            signal_clock_domain,
            signal_basis.current_tick().get(),
            signal_basis.last_advance_ordinal().get(),
            signal_clock_checkpoint,
            wake.wake_id().get(),
            wake.wake_ready_ordinal().get(),
            wake.wake_tick().get(),
        );
        let semantic_digest = Sha256::digest(semantic_basis.as_bytes());
        let full_basis = format!(
            "{semantic_basis}|truth-owner=worth-relational|signal-owner=worth-signal|bridge-owner=worth-runtime-bridge"
        );
        let digest = Sha256::digest(full_basis.as_bytes());
        Self {
            temporal_basis_identity: Arc::from(temporal_basis.identity().as_str().to_owned()),
            truth_basis_kind,
            truth_branch_identity,
            truth_snapshot_identity,
            truth_locator_identity,
            signal_clock_domain,
            signal_clock_tick: signal_basis.current_tick().get(),
            signal_clock_advance_ordinal: signal_basis.last_advance_ordinal().get(),
            signal_clock_checkpoint,
            wake_id: wake.wake_id().get(),
            wake_ready_ordinal: wake.wake_ready_ordinal().get(),
            wake_tick: wake.wake_tick().get(),
            truth_owner: Arc::from("worth-relational"),
            signal_owner: Arc::from("worth-signal"),
            bridge_owner: Arc::from("worth-runtime-bridge"),
            semantic_digest: Arc::from(format!(
                "bridge-temporal-async-certification-basis-semantic:sha256:{semantic_digest:x}"
            )),
            digest: Arc::from(format!(
                "bridge-temporal-async-certification-basis:sha256:{digest:x}"
            )),
        }
    }

    pub fn temporal_basis_identity(&self) -> &str {
        self.temporal_basis_identity.as_ref()
    }

    pub fn truth_owner(&self) -> &str {
        self.truth_owner.as_ref()
    }

    pub fn signal_owner(&self) -> &str {
        self.signal_owner.as_ref()
    }

    pub fn bridge_owner(&self) -> &str {
        self.bridge_owner.as_ref()
    }

    pub fn semantic_digest(&self) -> &str {
        self.semantic_digest.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
