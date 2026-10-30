use worth_foundational::{
    FoundationalBoundaryEvidenceCategory, FoundationalBoundaryEvidencePrimitiveDefinition,
};

fn main() {
    let _definition = FoundationalBoundaryEvidencePrimitiveDefinition::<
        FoundationalBoundaryEvidenceCategory,
    > {
        primitive: FoundationalBoundaryEvidenceCategory::Lineage,
        name: "lineage",
        intended_use: "bad",
        must_not_mean: "worse",
    };
}
