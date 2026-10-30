use worth_relational::facade::branch::RelationalBranchObservationAuthorityMarker;

fn main() {
    let marker = RelationalBranchObservationAuthorityMarker(());
    let _witness = worth_proof::AuthorityWitness::from_authority_marker(marker);
}
