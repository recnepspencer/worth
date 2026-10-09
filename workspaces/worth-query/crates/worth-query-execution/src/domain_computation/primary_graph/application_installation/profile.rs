/// Query-owned execution profile for an in-memory application.
///
/// The choice is explicit because application topology determines the bounded
/// validation and storage policy needed by its governed transactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorthQueryInMemoryApplicationProfile {
    GeneralPurpose,
    GeometryKernel,
    /// Finite 10k-node workflow publication over GeometryKernel policies.
    WorkflowScale,
}

impl WorthQueryInMemoryApplicationProfile {
    pub(super) fn publication_override(
        self,
        maximum_records: Option<std::num::NonZeroUsize>,
    ) -> Option<worth_relational::facade::config::PublicationConfig> {
        if maximum_records.is_none() && self != Self::WorkflowScale {
            return None;
        }
        let mut policy = worth_relational::facade::runtime::RelationalRuntimeConfig::resolved(
            self.relational_profile(),
            Default::default(),
        )
        .publication
        .policy;
        if self == Self::WorkflowScale {
            policy.max_patch_records_per_commit = 240_000;
            policy.max_prepared_root_bytes = 512 * 1024 * 1024;
        }
        if let Some(maximum) = maximum_records {
            policy.max_patch_records_per_commit = maximum.get();
        }
        Some(policy)
    }

    pub(super) const fn relational_profile(
        self,
    ) -> worth_relational::facade::config::RelationalRuntimeProfile {
        match self {
            Self::GeneralPurpose => {
                worth_relational::facade::config::RelationalRuntimeProfile::AiWorkflow
            }
            Self::GeometryKernel | Self::WorkflowScale => {
                worth_relational::facade::config::RelationalRuntimeProfile::GeometryKernel
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_record_limit_changes_no_other_profile_policy() {
        for profile in [
            WorthQueryInMemoryApplicationProfile::GeneralPurpose,
            WorthQueryInMemoryApplicationProfile::GeometryKernel,
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
                expected.max_patch_records_per_commit = maximum;
                assert_eq!(
                    profile.publication_override(std::num::NonZeroUsize::new(maximum)),
                    Some(expected)
                );
            }
        }
    }

    #[test]
    fn workflow_scale_has_finite_publication_retention_envelope() {
        let profile = WorthQueryInMemoryApplicationProfile::WorkflowScale;
        let publication = profile.publication_override(None).unwrap();
        assert_eq!(publication.max_patch_records_per_commit, 240_000);
        assert_eq!(publication.max_prepared_root_bytes, 512 * 1024 * 1024);
    }
}
