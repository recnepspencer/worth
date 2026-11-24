use worth_proof::{AuthorityMarker, AuthorityWitness};

pub struct RelationalSourceTruthAuthority {
    _owner_seal: (),
}

impl AuthorityMarker for RelationalSourceTruthAuthority {}

pub fn relational_source_truth_authority() -> AuthorityWitness<RelationalSourceTruthAuthority> {
    AuthorityWitness::from_authority_marker(RelationalSourceTruthAuthority { _owner_seal: () })
}
