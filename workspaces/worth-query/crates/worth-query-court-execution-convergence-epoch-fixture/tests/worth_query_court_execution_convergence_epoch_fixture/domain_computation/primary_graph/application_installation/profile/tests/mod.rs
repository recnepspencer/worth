use worth_query_execution::facade::application_installation::WorthQueryInMemoryApplicationProfile;

#[test]
fn explicit_record_limit_changes_no_other_profile_policy() {
    for profile in [
        WorthQueryInMemoryApplicationProfile::GeneralPurpose,
        WorthQueryInMemoryApplicationProfile::GeometryKernel,
        WorthQueryInMemoryApplicationProfile::WorkflowScale,
    ] {
        assert_eq!(profile.publication_override(None), None, "omission must preserve profile-default provenance, not install a numerically equal override");
        let default = worth_relational::facade::runtime::RelationalRuntimeConfig::resolved(
            profile.relational_profile(),
            Default::default(),
        )
        .publication
        .policy;
        for maximum in [1, 65_536] {
            let mut expected = default.clone();
            expected.max_patch_records_per_commit = Some(maximum);
            assert_eq!(
                profile.publication_override(std::num::NonZeroUsize::new(maximum)),
                Some(expected)
            );
        }
    }
}
