use worth_signal::facade::branch::SignalBranchBasisAuthorityMarker;

fn main() {
    let marker = SignalBranchBasisAuthorityMarker(());
    let _witness = worth_proof::AuthorityWitness::from_authority_marker(marker);
}
