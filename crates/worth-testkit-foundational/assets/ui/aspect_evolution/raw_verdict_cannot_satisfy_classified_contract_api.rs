use worth_foundational::{
    AspectEvolutionClassifiedContractArtifact, AspectEvolutionKind, AspectEvolutionVerdict,
};

fn requires_classified_evolution(_classified: AspectEvolutionClassifiedContractArtifact) {}

fn main() {
    let verdict = AspectEvolutionVerdict::new(AspectEvolutionKind::Widening, "raw verdict");
    requires_classified_evolution(verdict);
}
