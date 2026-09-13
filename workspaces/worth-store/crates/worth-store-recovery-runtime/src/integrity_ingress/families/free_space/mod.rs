mod header;
mod membership_block;

pub(in crate::integrity_ingress) use header::IntegrityAdmittedFreeSpaceHeader;
pub(in crate::integrity_ingress) use membership_block::IntegrityAdmittedFreeSpaceMembershipBlock;

#[cfg(test)]
pub(super) fn owner_valid_compile_contracts() {
    header::owner_valid_compile_contract();
    membership_block::owner_valid_compile_contract();
}
