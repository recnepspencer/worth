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
        staging: Option<super::WorthQueryTransactionStagingResources>,
    ) -> Option<worth_relational::facade::config::PublicationConfig> {
        if maximum_records.is_none() && staging.is_none() && self != Self::WorkflowScale {
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
            policy.max_transaction_footprint_loci = 240_000;
            policy.max_transaction_overlay_bytes = 256 * 1024 * 1024;
            policy.max_prepared_root_bytes = 512 * 1024 * 1024;
        }
        if let Some(maximum) = maximum_records {
            policy.max_patch_records_per_commit = maximum.get();
        }
        if let Some(staging) = staging {
            policy.max_transaction_overlay_bytes = staging.maximum_overlay_bytes();
            policy.max_transaction_footprint_loci = staging.maximum_footprint_loci();
        }
        Some(policy)
    }

    pub(super) fn relation_integrity_scope_budget(
        self,
    ) -> Option<worth_relational::facade::config::RelationIntegrityScopeBudget> {
        (self == Self::WorkflowScale).then(|| {
            let mut budget = worth_relational::facade::runtime::RelationalRuntimeConfig::resolved(
                self.relational_profile(),
                Default::default(),
            )
            .execution
            .relation_integrity_scope_budget;
            budget.max_planned_edges = 80_000;
            budget
        })
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
            assert_eq!(profile.publication_override(None, None), None, "omission must preserve profile-default provenance, not install a numerically equal override");
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
                    profile.publication_override(std::num::NonZeroUsize::new(maximum), None),
                    Some(expected)
                );
            }
        }
    }

    #[test]
    fn workflow_scale_has_finite_publication_and_integrity_envelopes() {
        let profile = WorthQueryInMemoryApplicationProfile::WorkflowScale;
        let publication = profile.publication_override(None, None).unwrap();
        assert_eq!(publication.max_patch_records_per_commit, 240_000);
        assert_eq!(publication.max_transaction_footprint_loci, 240_000);
        assert_eq!(publication.max_transaction_overlay_bytes, 256 * 1024 * 1024);
        assert_eq!(publication.max_prepared_root_bytes, 512 * 1024 * 1024);
        let integrity = profile.relation_integrity_scope_budget().unwrap();
        assert_eq!(integrity.max_planned_edges, 80_000);
        assert_eq!(integrity.max_touched_entities, 32_768);
        assert_eq!(integrity.max_scanned_relations, 131_072);
    }

    #[test]
    fn explicit_staging_resources_change_only_overlay_and_footprint() {
        use std::num::{NonZeroU64, NonZeroUsize};
        for profile in [
            WorthQueryInMemoryApplicationProfile::GeneralPurpose,
            WorthQueryInMemoryApplicationProfile::GeometryKernel,
            WorthQueryInMemoryApplicationProfile::WorkflowScale,
        ] {
            let baseline = profile.publication_override(None, None).unwrap_or_else(|| {
                worth_relational::facade::runtime::RelationalRuntimeConfig::resolved(
                    profile.relational_profile(),
                    Default::default(),
                )
                .publication
                .policy
            });
            for records in [None, NonZeroUsize::new(17)] {
                let staging = super::super::WorthQueryTransactionStagingResources::bounded(
                    NonZeroU64::new(1_048_576).unwrap(),
                    NonZeroUsize::new(128).unwrap(),
                );
                let mut expected = baseline.clone();
                expected.max_transaction_overlay_bytes = 1_048_576;
                expected.max_transaction_footprint_loci = 128;
                if let Some(records) = records {
                    expected.max_patch_records_per_commit = records.get();
                }
                assert_eq!(
                    profile.publication_override(records, Some(staging)),
                    Some(expected)
                );
            }
        }
    }
}
