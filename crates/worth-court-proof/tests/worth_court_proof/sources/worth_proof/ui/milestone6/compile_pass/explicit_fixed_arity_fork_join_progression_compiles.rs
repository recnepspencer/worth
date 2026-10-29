use worth_proof::{
    fork_artifact_pair, join_artifact_pair, Artifact, ForkOutputs2, JoinInputs2, NoAssumptionBasis,
    NoProofs, PhaseMarker,
};

struct RawPhase;
impl PhaseMarker for RawPhase {}

fn main() {
    let source = Artifact::<RawPhase, _>::new((3_u8, 5_u8));

    let forked = fork_artifact_pair(source, |payload, proofs, basis| {
        let _ = proofs;
        let _ = basis;
        ForkOutputs2::new(
            (payload.0, NoProofs, NoAssumptionBasis),
            (payload.1, NoProofs, NoAssumptionBasis),
        )
    });

    let (left, right) = forked.into_parts();
    let joined = join_artifact_pair(JoinInputs2::new(left, right), |left, right| {
        (left.0 + right.0, NoProofs, NoAssumptionBasis)
    });

    let _ = joined.payload();
}
