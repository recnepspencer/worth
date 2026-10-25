//! One authored policy composes paired edges with directed physical uniqueness.
use super::{ApplicationRelationIntegrity, ApplicationRelationUniquenessScope, LawPolicy};

// compiled-example: begin
const PAIRED_UNIQUE: ApplicationRelationIntegrity =
    ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
        .symmetric()
        .unique(ApplicationRelationUniquenessScope::DirectedSemanticEdge);
// compiled-example: end

#[derive(Clone, Debug)]
pub struct SymmetricDirectedUnique;
impl LawPolicy for SymmetricDirectedUnique {
    const INTEGRITY: ApplicationRelationIntegrity = PAIRED_UNIQUE;
}
