//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryReadFamily>(
            "workspaces/worth-query/crates/worth-query/src/runtime/surface/read_family.rs",
            "WorthQueryReadFamily",
            128 * 1024,
        );
        owner.field::<String>("family_name");
        owner.field::<String>("family_digest");
        owner.field::<WorthQueryReadFamilyAdmission>("admission");
        owner.field::<WorthQueryReadGraph>("read_graph");
    }
}
