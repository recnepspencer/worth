#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiViewportResizeEvidence {
    strategy: crate::runtime::UiViewportReceiptCommitStrategy,
    maximum_committed_receipts: u16,
    admitted_observations: u16,
    selected_neighborhoods: u16,
    committed_receipts: u16,
    durable_mutations: u16,
    authority_probes: u16,
    emitted_targets: u16,
    materialized_host_target_sets: u16,
    frame_epoch: crate::runtime::UiAllocationFrameEpoch,
    transaction_idempotency_key: u64,
    primary_neighborhood_identity_digest: u64,
    selected_neighborhood_identity_digests: Box<[u64]>,
    root_posture: crate::graph::UiReplanRootPosture,
}

impl UiViewportResizeEvidence {
    pub(crate) fn from_committed(
        basis: &crate::runtime::UiViewportResizeCommitBasis,
        committed: &crate::runtime::UiCommittedAllocationReplan,
    ) -> Self {
        let plan = basis.plan();
        let selection = basis.selection();
        Self {
            strategy: crate::runtime::UiViewportReceiptCommitStrategy::from_resolved_policy(
                plan.policy(),
            ),
            maximum_committed_receipts: plan.policy().budget().max_committed_receipts(),
            admitted_observations: u16::try_from(
                plan.narrowed_families()
                    .filter(|family| {
                        *family
                            == crate::runtime::UiAllocationInvalidationFamily::ViewportExtentChange
                    })
                    .count(),
            )
            .expect("a frame plan counts its invalidations in u16"),
            selected_neighborhoods: selection.neighborhood_count(),
            committed_receipts: committed.counters().committed_receipts(),
            durable_mutations: 0,
            authority_probes: plan.counters().authority_probes(),
            emitted_targets: plan.counters().emitted_targets(),
            materialized_host_target_sets: plan.counters().materialized_host_target_sets(),
            frame_epoch: committed.transaction().frame_epoch(),
            transaction_idempotency_key: committed.transaction().idempotency_key(),
            primary_neighborhood_identity_digest: selection.primary().identity().identity_digest(),
            selected_neighborhood_identity_digests: selection
                .ordered_neighborhoods()
                .iter()
                .map(|item| item.identity().identity_digest())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            root_posture: selection.root_posture(),
        }
    }
    pub fn strategy(&self) -> crate::runtime::UiViewportReceiptCommitStrategy {
        self.strategy
    }
    pub fn maximum_committed_receipts(&self) -> u16 {
        self.maximum_committed_receipts
    }
    pub fn admitted_observations(&self) -> u16 {
        self.admitted_observations
    }
    pub fn selected_neighborhoods(&self) -> u16 {
        self.selected_neighborhoods
    }
    pub fn committed_receipts(&self) -> u16 {
        self.committed_receipts
    }
    pub fn durable_mutations(&self) -> u16 {
        self.durable_mutations
    }
    pub fn authority_probes(&self) -> u16 {
        self.authority_probes
    }
    pub fn emitted_targets(&self) -> u16 {
        self.emitted_targets
    }
    pub fn materialized_host_target_sets(&self) -> u16 {
        self.materialized_host_target_sets
    }
    pub fn frame_epoch(&self) -> crate::runtime::UiAllocationFrameEpoch {
        self.frame_epoch
    }
    pub fn transaction_idempotency_key(&self) -> u64 {
        self.transaction_idempotency_key
    }
    pub fn primary_neighborhood_identity_digest(&self) -> u64 {
        self.primary_neighborhood_identity_digest
    }
    pub fn selected_neighborhood_identity_digests(&self) -> &[u64] {
        &self.selected_neighborhood_identity_digests
    }
    pub fn root_posture(&self) -> crate::graph::UiReplanRootPosture {
        self.root_posture
    }
}
