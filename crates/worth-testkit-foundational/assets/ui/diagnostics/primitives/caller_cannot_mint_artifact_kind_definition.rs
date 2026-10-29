use worth_foundational::{
    FoundationalDiagnosticArtifactKind, FoundationalDiagnosticArtifactKindDefinition,
};

fn main() {
    let _definition = FoundationalDiagnosticArtifactKindDefinition::new(
        FoundationalDiagnosticArtifactKind::Summary,
        "summary",
        "Worthd meaning",
        "nothing",
    );
}
