//! A query and its managed-index views use the same admitted schema allowance.
use crate::installation::managed_index_parent_profiles::{bind, model, queries};

#[test]
fn admitted_query_meaning_outgrows_the_former_fixed_installation_byte_ceiling() {
    type Schema = model::ParentSchema<9>;
    let schema = bind::<9>().expect("admitted query and managed-index meaning installs");
    let binding = schema
        .installed_query_binding::<queries::SourceBinding<Schema>>()
        .unwrap();
    let query = binding.query();
    assert!(query.canonical_basis().work().canonical_encoded_bytes() > 1024 * 1024);
    assert!(
        query
            .canonical_work_policy()
            .installation()
            .maximum_encoded_bytes()
            > 1024 * 1024
    );
    assert_eq!(
        query
            .canonical_work_policy()
            .admission_planning()
            .maximum_entry_count(),
        4096,
    );
}

pub(crate) fn identity_output() -> &'static str {
    static OUTPUT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    OUTPUT.get_or_init(|| "identity".repeat(160_000)).as_str()
}
