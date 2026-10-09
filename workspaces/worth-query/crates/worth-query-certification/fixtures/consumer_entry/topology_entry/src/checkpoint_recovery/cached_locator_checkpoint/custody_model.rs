//! Independent custody budgets for one root and a pressured root/final pair.
//! registry/prerequisite_claims.rs:136-240 charges exact grown settlement capacity;
//! max(2c,4) is an admission projection, not retained slots.
use super::*;
use worth_query_host::facade::application_contribution::WorthQueryApplicationProducerBinding;
use worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile;

struct Custody {
    layout: primary_graph::RequiredCustodyLayoutForTest,
    root_name: usize,
    final_name: usize,
}
impl Custody {
    fn new() -> Self {
        Self {
            layout: primary_graph::required_custody_layout_for_test(),
            root_name: crate::producer::InitialPlanarProducer::<CheckpointSchema>::IDENTITY.len(),
            final_name:
                crate::final_output::PlanarFinalOutputProducer::<CheckpointSchema>::IDENTITY.len(),
        }
    }
    fn postings(&self, sources: usize) -> usize {
        assert!((1..=4).contains(&sources));
        self.layout.outer_posting_node
            + sources * (self.layout.inner_posting_node + self.layout.posting)
    }
    fn root_row(&self) -> usize {
        self.layout.ready
            + self.layout.source
            + self.layout.context_without_producer
            + self.root_name
    }
    fn root_member(&self) -> usize {
        self.layout.member_without_producer + self.root_name
    }
    fn open_root(&self) -> usize {
        self.root_row() + self.root_member() + self.postings(1) + self.layout.settlement_slot
    }
    fn closed_pair(&self) -> usize {
        2 * (self.layout.ready + self.layout.source + self.layout.context_without_producer)
            + self.root_name
            + self.final_name
            + self.root_member()
            + self.postings(2)
            + 2 * self.layout.settlement_slot
            + self.layout.prerequisite_slot
    }
    fn three_row_peak(&self) -> usize {
        self.closed_pair() + self.root_row() + self.root_member() + self.postings(3)
            - self.postings(2)
            + self.layout.settlement_slot
    }
    fn assert_pressure(&self, budget: usize) {
        type Runtime = primary_graph::WorthQueryPrimaryGraphApplicationRuntime<CheckpointSchema>;
        let initial_pair_peak = self.closed_pair()
            + self.layout.member_without_producer
            + self.final_name
            + 3 * self.layout.settlement_slot
            + Runtime::required_continuation_slot_custody_bytes_for_test()
            + Runtime::required_successor_custody_bytes_for_test::<
                crate::final_output::PlanarFinalOutputFamily,
            >()
            + primary_graph::required_handoff_custody_bytes_for_test(
                crate::final_output::PlanarFinalOutputProducer::<CheckpointSchema>::IDENTITY,
            );
        assert!(
            initial_pair_peak <= budget,
            "the initial pair fits its owner-quoted budget"
        );
        let four_closed = self.closed_pair()
            + 2 * (self.root_row() + self.layout.settlement_slot)
            + self.postings(4)
            - self.postings(2);
        assert!(
            four_closed > budget,
            "the fourth closed publication requires reclamation"
        );
    }
}
fn profile(bytes: usize) -> WorthQueryOutputDemandResourceProfile {
    WorthQueryOutputDemandResourceProfile::standard()
        .with_registry_required_retained_bytes(NonZeroUsize::new(bytes).unwrap())
}
pub(super) fn single_root_profile() -> WorthQueryOutputDemandResourceProfile {
    profile(Custody::new().open_root())
}
pub(super) fn reclamation_profile() -> WorthQueryOutputDemandResourceProfile {
    let custody = Custody::new();
    let budget = custody.three_row_peak();
    custody.assert_pressure(budget);
    profile(budget)
}

pub(super) fn report_models() {
    let model = Custody::new();
    let l = &model.layout;
    for (label, roots, finals, root_members, final_members) in [
        ("ROOT_OPEN", 1, 0, 1, 0),
        ("ROOT_CLOSED", 1, 0, 0, 0),
        ("PAIR_OPEN", 1, 1, 1, 1),
        ("PAIR_CLOSED", 1, 1, 1, 0),
    ] {
        eprintln!(
            "CACHED_MODEL {label} {:?}",
            [
                (
                    "members",
                    root_members * model.root_member()
                        + final_members * (l.member_without_producer + model.final_name)
                ),
                ("settlement_slots", (roots + finals) * l.settlement_slot),
                (
                    "execution_contexts",
                    roots * (l.context_without_producer + model.root_name)
                        + finals * (l.context_without_producer + model.final_name)
                ),
                ("prerequisite_slots", finals * l.prerequisite_slot),
                ("settlement_postings", model.postings(roots + finals)),
                (
                    "shared_ready_source_continuation",
                    (roots + finals) * (l.ready + l.source)
                ),
            ]
        );
    }
    eprintln!("CACHED_MODEL_CAPACITY H1={} H2={} H3={} units R={} S={} M_A={} M_F={} K_A={} K_F={} Q={} E={}",
        model.open_root(), model.closed_pair(), model.three_row_peak(), l.ready, l.source,
        model.root_member(), l.member_without_producer + model.final_name,
        l.context_without_producer + model.root_name, l.context_without_producer + model.final_name,
        l.settlement_slot, l.prerequisite_slot);
}
