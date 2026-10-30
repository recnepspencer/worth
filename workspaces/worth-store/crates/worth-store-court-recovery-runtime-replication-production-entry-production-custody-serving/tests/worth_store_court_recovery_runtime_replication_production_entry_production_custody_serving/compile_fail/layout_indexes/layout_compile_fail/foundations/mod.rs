#[test]
fn raw_physical_references_cannot_admit_index_materialization() {
    crate::compile_fail_support::assert_compile_fails(
        "raw_physical_reference_cannot_admit_btree_materialization.rs",
        &[
            "expected `RootPublicationValidationWitness`",
            "found `PhysicalReference`",
        ],
        &["worth_store_physical_format"],
    );
}
