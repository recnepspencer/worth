use worth_foundational::{CanonicalBasisDomain, CanonicalBasisEntryId};

fn requires_entry_id(_: CanonicalBasisEntryId<CanonicalBasisDomain>) {}

fn main() {
    requires_entry_id(7_u32);
}
