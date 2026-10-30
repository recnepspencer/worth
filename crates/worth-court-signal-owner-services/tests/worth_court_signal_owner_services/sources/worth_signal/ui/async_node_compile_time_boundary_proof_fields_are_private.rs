use worth_signal::facade::core::AsyncNodeCompileTimeBoundaryProof;

fn fake<T>() -> T {
    panic!("type-check only")
}

fn main() {
    let proof: AsyncNodeCompileTimeBoundaryProof = fake();
    let _ = proof.fixture_labels;
}
