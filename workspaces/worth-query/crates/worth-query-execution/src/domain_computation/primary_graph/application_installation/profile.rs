/// Query-owned execution profile for an in-memory application.
///
/// The choice is explicit because application topology determines the bounded
/// validation and storage policy needed by its governed transactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorthQueryInMemoryApplicationProfile {
    GeneralPurpose,
    GeometryKernel,
    /// 10k-node workflow workload over GeometryKernel policies.
    WorkflowScale,
}

impl WorthQueryInMemoryApplicationProfile {
    pub fn publication_override(
        self,
        maximum_records: Option<std::num::NonZeroUsize>,
    ) -> Option<worth_relational::facade::config::PublicationConfig> {
        let maximum = maximum_records?;
        let mut policy = worth_relational::facade::runtime::RelationalRuntimeConfig::resolved(
            self.relational_profile(),
            Default::default(),
        )
        .publication
        .policy;
        policy.max_patch_records_per_commit = Some(maximum.get());
        Some(policy)
    }

    pub const fn relational_profile(
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
