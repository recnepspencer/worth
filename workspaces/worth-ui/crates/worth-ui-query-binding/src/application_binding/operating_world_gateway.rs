use worth_query::facade::{foundation::ObservationLaneWitness, installed, runtime};

use crate::{
    application_binding::WorthUiInstalledSnapshotOperationReference,
    WorthUiInstalledQueryBindingReference, WorthUiQueryHost, WorthUiQueryInstallationDenial,
    WorthUiSnapshotMeasurement, WorthUiSnapshotMeasurementFamily,
};

pub type WorthUiBoundSnapshotMeasurement<L> = installed::WorthQueryBoundDomainOperation<
    crate::WorthUiDomainEntry,
    WorthUiSnapshotMeasurement,
    WorthUiSnapshotMeasurementFamily,
    L,
>;

#[derive(Debug)]
pub enum WorthUiQueryOperationAttemptDenial {
    Installation(WorthUiQueryInstallationDenial),
    InstalledDomainAuthorityMismatch,
    OperatingWorld(Box<installed::WorthQueryOperatingWorldEntryDenial>),
}

/// One attempt-scoped entry into Query's installed operating world.
///
/// Construction verifies that the application-owned reference and the
/// workspace resolve the same exact installed-domain authority. Consuming the
/// gateway binds one operation; neither the world nor the move-only bound
/// operation can be retained in an application plan or steady frame.
pub struct WorthUiQueryOperatingWorldGateway<'runtime> {
    world: installed::WorthQueryInstalledOperatingWorld<'runtime, ObservationLaneWitness>,
    pub(super) reference: WorthUiInstalledQueryBindingReference,
    operation: WorthUiInstalledSnapshotOperationReference,
}

impl WorthUiInstalledQueryBindingReference {
    pub fn enter_snapshot_attempt<'runtime>(
        &self,
        workspace: &'runtime runtime::WorthQueryWorkspace,
    ) -> Result<WorthUiQueryOperatingWorldGateway<'runtime>, WorthUiQueryOperationAttemptDenial>
    {
        let operation = self.snapshot_operation();
        let current = WorthUiQueryHost::from_workspace(workspace)
            .installed_domain()
            .map_err(WorthUiQueryOperationAttemptDenial::Installation)?;
        if !current.shares_authority_with(self.installed_domain()) {
            return Err(WorthUiQueryOperationAttemptDenial::InstalledDomainAuthorityMismatch);
        }
        Ok(WorthUiQueryOperatingWorldGateway {
            world: workspace
                .observe_operating_world(workspace.current_world())
                .map_err(|denial| {
                    WorthUiQueryOperationAttemptDenial::OperatingWorld(Box::new(denial))
                })?,
            reference: self.clone(),
            operation,
        })
    }
}

impl WorthUiQueryOperatingWorldGateway<'_> {
    pub fn bind_snapshot(
        self,
    ) -> Result<
        WorthUiBoundSnapshotMeasurement<ObservationLaneWitness>,
        Box<installed::WorthQueryOperationBindingDenial>,
    > {
        self.world
            .family(self.operation.family)
            .bind(
                self.reference.installed_domain().handle(),
                self.operation.operation,
            )
            .map_err(Box::new)
    }
}
