//! Sealed aftermath-installation source used by classification fixtures.

use worth_query_declaration::facade::application_aftermath::PortableApplicationAftermathContract;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationDecisionReadTarget, ApplicationSchemaBindingIdentity,
};

use crate::application_aftermath::InstalledExternalEffectContract;
use crate::package::WorthQueryPortableInstalledReconciliationProcedureRecord;

pub(super) struct FixtureAftermathInstallationSource {
    binding: ApplicationSchemaBindingIdentity,
    operation: String,
    decision_reads: Vec<ApplicationOperationDecisionReadTarget>,
    external_effect: InstalledExternalEffectContract,
    portable_aftermath: Option<PortableApplicationAftermathContract>,
    portable_reconciliation: Option<WorthQueryPortableInstalledReconciliationProcedureRecord>,
}

impl FixtureAftermathInstallationSource {
    pub(super) fn new(
        binding: ApplicationSchemaBindingIdentity,
        operation: String,
        decision_reads: Vec<ApplicationOperationDecisionReadTarget>,
        external_effect: InstalledExternalEffectContract,
        portable_aftermath: Option<PortableApplicationAftermathContract>,
        portable_reconciliation: Option<WorthQueryPortableInstalledReconciliationProcedureRecord>,
    ) -> Self {
        Self {
            binding,
            operation,
            decision_reads,
            external_effect,
            portable_aftermath,
            portable_reconciliation,
        }
    }
}

impl super::super::aftermath_installation_source_seal::Sealed
    for FixtureAftermathInstallationSource
{
}

impl super::super::WorthQueryOperationAftermathInstallationSource
    for FixtureAftermathInstallationSource
{
    fn binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding
    }

    fn operation(&self) -> &str {
        &self.operation
    }

    fn portable_decision_reads(&self) -> Vec<ApplicationOperationDecisionReadTarget> {
        self.decision_reads.clone()
    }

    fn external_effect(&self) -> &InstalledExternalEffectContract {
        &self.external_effect
    }

    fn portable_aftermath(&self) -> Option<&PortableApplicationAftermathContract> {
        self.portable_aftermath.as_ref()
    }

    fn portable_reconciliation(
        &self,
    ) -> Option<&WorthQueryPortableInstalledReconciliationProcedureRecord> {
        self.portable_reconciliation.as_ref()
    }
}
