use worth_proof::{fork_artifact_pair, Artifact, ForkOutputs2, PhaseMarker};

struct RawPhase;
impl PhaseMarker for RawPhase {}

fn main() {
    let artifact = Artifact::<RawPhase, _>::new((3_u8, 5_u8));

    let _forked = fork_artifact_pair(artifact, |payload, proofs, basis| {
        ((payload.0, proofs, basis), (payload.1, worth_proof::NoProofs, worth_proof::NoAssumptionBasis))
    });

    let _unused = ForkOutputs2::new;
}
