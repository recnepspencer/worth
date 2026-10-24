use worth_relational::facade::durability::{
    RelationalCanonicalCommitArtifact, RelationalDurableMedium, RelationalDurablePublication,
};
use worth_relational::facade::runtime::RelationalRuntimeApi;

#[derive(Debug)]
struct Journal;

impl RelationalDurableMedium for Journal {
    fn publish(&self, _: &RelationalCanonicalCommitArtifact) -> RelationalDurablePublication {
        RelationalDurablePublication::Committed
    }
}

fn main() {
    let medium: Box<dyn RelationalDurableMedium> = Box::new(Journal);
    let builder = RelationalRuntimeApi::builder();
    let _first = builder.clone().build_durable(medium);
    let _second = builder.build_durable(medium);
}
