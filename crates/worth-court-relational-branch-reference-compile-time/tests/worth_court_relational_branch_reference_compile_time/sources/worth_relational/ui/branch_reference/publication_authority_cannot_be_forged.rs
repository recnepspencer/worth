use worth_relational::facade::branch::RelationalBranchPublicationAuthorityMarker;

fn main() {
    let marker = RelationalBranchPublicationAuthorityMarker(());
    let _witness = worth_proof::AuthorityWitness::from_authority_marker(marker);
    let _sealed = RelationalBranchPublicationAuthorityMarker::seal();
    let _issued = RelationalBranchPublicationAuthorityMarker::witness();
}
