use worth_proof::{Artifact, NoAssumptionBasis, NoProofs, PhaseMarker};

struct RawPhase;
impl PhaseMarker for RawPhase {}

struct ValidatedPhase;
impl PhaseMarker for ValidatedPhase {}

fn requires_validated<T>(_: &Artifact<ValidatedPhase, T, NoProofs, NoAssumptionBasis>) {}

fn main() {
    let raw = Artifact::<RawPhase, _>::new("payload");
    requires_validated(&raw);
}
