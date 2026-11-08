//! Install the original recorded root from concrete component-owner admissions.
use super::{preparation::ReadmittedRootPreparation, *};
use crate::durability::{DecodedWorldRecord, WorldReadmissionDenial as Denial, WorldRecord};
use crate::identity::readmission::{readmit_bootstrap, readmit_commit, readmit_incarnation};
use worth_relational::facade::durability::RecoveredRelationalBranchBasis;
use worth_runtime_bridge::facade::AdmittedRuntimeWorldCorrespondenceBasis;
use worth_signal::facade::branch::AdmittedSignalInitialEmptyRoot;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The named factory obtains these three real admissions in owner order.
    /// Native is deliberately historical: comparing it to latest would replace
    /// the product's original predecessor after an unpublished Native effect.
    pub(crate) fn install_readmitted_root(
        &self,
        artifact: &DecodedWorldRecord,
        native: RecoveredRelationalBranchBasis,
        signal: AdmittedSignalInitialEmptyRoot,
        bridge: AdmittedRuntimeWorldCorrespondenceBasis,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            ProductBranchObservation,
            crate::durability::WorldCheckpointOrigin,
        ),
        Denial,
    > {
        let WorldRecord::Branch {
            head,
            bootstrap,
            components,
            ..
        } = &artifact.record
        else {
            unreachable!("owner readmission selected the original Branch frame")
        };
        let creation = crate::branch::ProductBranchCreationIntent::named(head.branch.name())
            .map_err(|_| Denial::BranchLineage)?;
        let intent = RuntimeWorldBootstrapIntent::recovered(
            creation,
            native,
            signal.basis().clone(),
            bridge,
        );
        let basis = crate::basis::admit_validated(
            &self
                .state
                .identities
                .lock()
                .unwrap_or_else(|e| e.into_inner()),
            intent.relational_basis().clone(),
            intent.signal_basis().clone(),
            intent.correspondence_basis().clone(),
        );
        let retained = basis.clone();
        let result = (|| {
            let original = self
                .state
                .relational_services()
                .basis_port()
                .export_recorded_retained_component(execution, basis.relational_basis())
                .map_err(Denial::NativeRecorded)?;
            if original != components.relational || signal.recorded() != &components.signal {
                return Err(Denial::BranchLineage);
            }
            let owner = self.owner_identity();
            let readmitted = ReadmittedRootPreparation {
                bootstrap: readmit_bootstrap(owner, *bootstrap).ok_or(Denial::ForeignOwner)?,
                branch: crate::identity::ProductBranchIdentity::issued(
                    owner,
                    intent.creation().name().clone(),
                ),
                incarnation: readmit_incarnation(owner, head.incarnation)
                    .ok_or(Denial::ForeignOwner)?,
                commit: readmit_commit(owner, head.commit).ok_or(Denial::ForeignOwner)?,
                basis,
            };
            let mut prepared = self
                .prepare_readmitted_root(intent, readmitted)
                .map_err(Denial::RootAdmission)?;
            // The branch frame was already selected through actual Home
            // recovery; installing its SAME identities performs no new I/O.
            let performed = prepared.install().map_err(Denial::RootAdmission)?;
            let origin = prepared
                .checkpoint_origin
                .take()
                .expect("same prepared admitted origin obligation");
            *self
                .state
                .bootstrap
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = RuntimeWorldBootstrapState::Performed;
            Ok((performed.product_branch().clone(), origin))
        })();
        result.map_err(|cause| {
            Denial::RootPreparation(Box::new(crate::durability::RefusedWorldRootReadmission {
                cause: Box::new(cause),
                basis: retained,
            }))
        })
    }
}
