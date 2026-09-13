pub(super) mod bootstrap;
pub(super) mod checkpoint;
pub(super) mod extent;
pub(super) mod free_space;
pub(super) mod page;
pub(super) mod root;
pub(super) mod segment_membership;
pub(super) mod wal;

#[cfg(test)]
pub(super) fn owner_valid_compile_contracts() {
    bootstrap::owner_valid_compile_contract();
    root::owner_valid_compile_contracts();
    segment_membership::owner_valid_compile_contract();
    free_space::owner_valid_compile_contracts();
    page::owner_valid_compile_contract();
    extent::owner_valid_compile_contracts();
    checkpoint::owner_valid_compile_contracts();
}
