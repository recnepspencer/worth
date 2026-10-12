//! Independent named arithmetic for the fixed advancement-custody fixture.
use crate::domain_computation::primary_graph::tests::fixture::{
    ExternalIdentityField, MappingStatusField, PrincipalIdentityField,
};
const KEYED_RECORD_WORK: u64 = 1; // read_records/read_prices.rs
pub(super) fn principal_and_commit_reads() -> u64 {
    let identity = ExternalIdentityField::reference();
    let status = MappingStatusField::reference();
    let principal = PrincipalIdentityField::reference();
    let revision = |aspect: &str, field: &str, struct_fields: u64, revision_levels: u64| {
        // Entry, u32 partition path, slot/kind, three eleven-key catalog
        // levels, declared binding/text, revisions, two symbol hashes,
        // revision-tree descent and creation metadata (7155ffdf95 prices).
        1 + (32 + 1)
            + 3
            + 3 * 11
            + (2 + aspect.len() as u64 + (struct_fields + 1) * (field.len() as u64 + 1))
            + 2
            + (aspect.len() + field.len()) as u64
            + 2
            + revision_levels * 11
            + 1
    };
    let one_principal = revision(identity.aspect(), identity.field(), 2, 2)
        + revision(status.aspect(), status.field(), 2, 2)
        + revision(principal.aspect(), principal.field(), 1, 1);
    assert_eq!(one_principal, 218 + 206 + 190);
    // Five principal readmissions: mapping + principal entity and one target
    // relation each. Four commit credential/read-set probes and eight index
    // before/after probes account for the remaining keyed entity reads.
    let principal_records = 5 * (2 + 1) * KEYED_RECORD_WORK;
    let commit_records = (4 + 8) * KEYED_RECORD_WORK;
    5 * one_principal + principal_records + commit_records
}
pub(super) const fn leased_native_staging() -> u64 {
    // The leased packet lane additionally copies/prepares 104 units, runs
    // four packet entries, 38 main-thread and 352 worker kernel units,
    // eleven validation reductions, and six preparation merge units.
    // The direct unleased lane does not stage these artifacts.
    104 + 4 + 38 + 352 + 11 + (1 + 2 + 1 + 2)
}
