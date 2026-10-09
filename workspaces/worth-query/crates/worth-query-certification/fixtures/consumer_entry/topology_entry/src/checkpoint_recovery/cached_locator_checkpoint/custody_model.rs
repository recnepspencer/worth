//! Independent custody budgets for one root and a pressured root/final pair.
//! registry/prerequisite_claims.rs:136-240 charges exact grown settlement capacity;
//! max(2c,4) is an admission projection, not retained slots.
use super::*;
use worth_query_host::facade::application_contribution::WorthQueryApplicationProducerBinding;

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
    fn root_member(&self) -> usize {
        self.layout.member_without_producer + self.root_name
    }
}

pub(super) fn expected_classes(
    roots: usize,
    finals: usize,
    root_members: usize,
    final_members: usize,
) -> Vec<(&'static str, usize)> {
    let model = Custody::new();
    let l = &model.layout;
    vec![
        (
            "members",
            root_members * model.root_member()
                + final_members * (l.member_without_producer + model.final_name),
        ),
        ("settlement_slots", (roots + finals) * l.settlement_slot),
        (
            "execution_contexts",
            roots * (l.context_without_producer + model.root_name)
                + finals * (l.context_without_producer + model.final_name),
        ),
        ("prerequisite_slots", 0),
        ("settlement_postings", model.postings(roots + finals)),
        ("pending_cleanup_keys", 0),
        ("live_preparation", 0),
        (
            "shared_ready_source_continuation",
            (roots + finals) * (l.ready + l.source),
        ),
    ]
}
