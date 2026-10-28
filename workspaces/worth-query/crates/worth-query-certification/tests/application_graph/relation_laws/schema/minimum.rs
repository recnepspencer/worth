use super::*;
// compiled-example: begin
const MINIMUM_TWO: ApplicationRelationIntegrity =
    ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
        .reaches_at_least(2, &["Node"]);
// compiled-example: end

#[derive(Clone, Debug)]
pub struct MinimumTwo;
impl LawPolicy for MinimumTwo {
    const INTEGRITY: ApplicationRelationIntegrity = MINIMUM_TWO;
}
#[derive(Clone, Debug)]
pub struct LeafMinimumTwo;
impl LawPolicy for LeafMinimumTwo {
    const INTEGRITY: ApplicationRelationIntegrity = Unrestricted::INTEGRITY;
    const FEEDS_INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            .reaches_at_least(2, &[Leaf::reference::<GraphSchema<Self>>().name()]);
}

#[derive(Clone, Debug)]
pub struct LeafAndPrincipalMinimumTwo;
impl LawPolicy for LeafAndPrincipalMinimumTwo {
    const INTEGRITY: ApplicationRelationIntegrity = Unrestricted::INTEGRITY;
    const FEEDS_INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            .reaches_at_least(2, &["Principal", "Leaf", "Leaf"]);
}
