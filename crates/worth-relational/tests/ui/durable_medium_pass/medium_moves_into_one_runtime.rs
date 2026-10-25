use worth_relational::facade::durability::{
    RelationalBranchTransitionArtifact, RelationalCanonicalCommitArtifact, RelationalDurableMedium,
    RelationalDurablePublication,
};
use worth_relational::facade::runtime::RelationalRuntimeApi;

#[derive(Debug)]
struct Journal;

impl RelationalDurableMedium for Journal {
    fn read_transition(
        &self,
        sequence: worth_relational::facade::branch::RelationalBranchTransitionSequence,
    ) -> Result<
        RelationalBranchTransitionArtifact,
        worth_relational::facade::durability::RelationalDurableTransitionReadFailure,
    > {
        Err(
            worth_relational::facade::durability::RelationalDurableTransitionReadFailure::Missing(
                sequence,
            ),
        )
    }

    fn publish(&self, _: &RelationalCanonicalCommitArtifact) -> RelationalDurablePublication {
        RelationalDurablePublication::Committed
    }

    fn publish_transition(
        &self,
        _: &RelationalBranchTransitionArtifact,
    ) -> RelationalDurablePublication {
        RelationalDurablePublication::Committed
    }
}

fn main() {
    let builder = RelationalRuntimeApi::builder();
    // A builder still clones: it never holds a medium.
    let _memory_only = builder.clone().build();
    let _durable = builder.build_durable(Box::new(Journal));
}
