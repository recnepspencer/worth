worth_proof::authority_marker!(pub ContractAuthority);

pub(crate) fn witness() -> worth_proof::AuthorityWitness<ContractAuthority> {
    ContractAuthority::witness()
}
