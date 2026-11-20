//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryExecutionRuntime {
    #[doc(hidden)]
    pub fn bind_standalone_test_domain_operation_for_test<L: BasisOperationLane>(
        &self,
        installation_authority: &WorthQueryExecutionInstallationAuthority,
        operation: &WorthQueryInstalledDomainOperationAuthority,
        basis: &AdmittedBasisCapability<L>,
        graph_authorities: &[&WorthQueryInstalledGraphParticipationAuthority],
        required_domains: &[(&str, &WorthQueryInstalledPackageAuthority)],
        commit_posture: WorthQueryExecutionCommitPosture,
        installed_support: WorthQueryInstalledOperationExecutionSupport,
    ) -> Result<WorthQueryExecutionBoundOperationAuthority, WorthQueryExecutionOperationBindingDenial>
    {
        Self::bind_standalone_test_domain_operation::<L>(
            self,
            installation_authority,
            operation,
            basis,
            graph_authorities,
            required_domains,
            commit_posture,
            installed_support,
        )
    }
}
