use std::marker::PhantomData;

use worth_relational::facade::branch::{
    RelationalBranchPublicationAuthority, RelationalBranchPublicationAuthorityMarker,
};

fn accept_publication_authority(_: RelationalBranchPublicationAuthority) {}

fn name_publication_authority_marker(
    _: PhantomData<RelationalBranchPublicationAuthorityMarker>,
) {
}

fn main() {
    let _ = accept_publication_authority;
    name_publication_authority_marker(PhantomData);
}
