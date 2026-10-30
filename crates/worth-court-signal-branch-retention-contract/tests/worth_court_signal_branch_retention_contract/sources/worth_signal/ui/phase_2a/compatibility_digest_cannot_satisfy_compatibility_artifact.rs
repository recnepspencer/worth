use worth_signal::facade::adapters::ScopedMergeProofPacket;

fn require_scoped_merge_proof(_: ScopedMergeProofPacket) {}

fn main() {
    let digest = String::from("signal-scoped-merge-proof-digest");
    require_scoped_merge_proof(digest);
}
