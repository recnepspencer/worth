use worth_proof::{join_artifact_pair, Artifact, JoinInputs2, PhaseMarker};

struct LoweredPhase;
impl PhaseMarker for LoweredPhase {}

fn main() {
    let left = Artifact::<LoweredPhase, _>::new(1_u8);
    let right = Artifact::<LoweredPhase, _>::new(2_u8);

    let _joined = join_artifact_pair(
        vec![left, right],
        |left: (u8, _, _), right: (u8, _, _)| (left.0 + right.0, left.1, left.2),
    );

    let _unused = JoinInputs2::new;
}
