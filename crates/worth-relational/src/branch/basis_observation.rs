use std::sync::Arc;

use super::{
    authority::issue_relational_branch_observation_authority,
    basis_axis_validation::require_root_matches_reference,
    basis_identity_validation::require_local_branch_identity, AdmittedRelationalBranchBasis,
    AdmittedRelationalBranchBasisInner, RelationalBranchBasisDenial,
    RelationalBranchBasisDescriptor, RelationalBranchBasisPosture, RelationalBranchIdentity,
    RelationalBranchRoot,
};
use crate::runtime::RelationalRuntime;

pub(crate) fn issue_admitted_relational_branch_basis_with_retention(
    descriptor: RelationalBranchBasisDescriptor,
    identity: RelationalBranchIdentity,
    root: Arc<RelationalBranchRoot>,
    publication_cell: super::RelationalBranchPublicationCell,
    retention_binding: &crate::history::retention::RelationalBranchRetentionBinding,
) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
    let retention = crate::history::retention::RelationalObservationRetentionObligation::acquire(
        retention_binding,
        Arc::clone(&root),
    )
    .map_err(map_retention_denial)?;
    Ok(AdmittedRelationalBranchBasis {
        inner: Arc::new(AdmittedRelationalBranchBasisInner {
            descriptor,
            identity,
            admission_identity: super::RelationalBranchBasisAdmissionIdentity::issue(),
            root,
            _authority: issue_relational_branch_observation_authority(),
            retention,
            retention_binding: retention_binding.clone(),
            publication_cell,
            registry_lease: std::sync::OnceLock::new(),
        }),
    })
}

impl RelationalRuntime {
    /// Observe and admit one exact owner branch basis from one cell snapshot.
    ///
    /// The pair returned is deliberately two values with different authority.
    /// The [`RelationalBranchBasisDescriptor`] is serializable, transportable
    /// evidence that describes an exact observed target and generation; it is
    /// data, not permission, and must pass `readmit_branch_basis` after it
    /// crosses a trust boundary. The [`AdmittedRelationalBranchBasis`] is the
    /// owner-issued authority that opens governed reads and transactions here
    /// and now.
    ///
    /// Reads bind to the basis's exact immutable observation, so a later
    /// movement of this branch does not change what the snapshot sees.
    ///
    /// ```
    /// use worth_relational::facade::runtime::RelationalRuntimeApi;
    /// use worth_relational::facade::schema::RelationalSchemaRegistry;
    ///
    /// let runtime = RelationalRuntimeApi::builder()
    ///     .schema_registry(RelationalSchemaRegistry::new())
    ///     .build();
    ///
    /// let identity = runtime.main_branch_identity();
    /// let (_descriptor, basis) = runtime
    ///     .observe_branch(&identity)
    ///     .expect("an owner-issued identity observes its own branch");
    ///
    /// let pinned = runtime
    ///     .snapshots()
    ///     .snapshot_for_observation(&basis.observation())
    ///     .expect("an exact observation opens a pinned read");
    /// runtime
    ///     .snapshots()
    ///     .release_snapshot(&pinned)
    ///     .expect("a pinned snapshot releases exactly once");
    /// ```
    ///
    /// The write progression that starts from this basis is on
    /// [`RelationalRuntime::preparation_port`]. The runnable owner workflow is
    /// `examples/branch_local_mvcc.rs`.
    pub fn observe_branch(
        &self,
        identity: &RelationalBranchIdentity,
    ) -> Result<
        (
            RelationalBranchBasisDescriptor,
            AdmittedRelationalBranchBasis,
        ),
        RelationalBranchBasisDenial,
    > {
        self.observe_branch_with_control(
            identity,
            &crate::mvcc::RelationalOperationControl::uninterrupted(),
        )
    }

    pub fn observe_branch_with_control(
        &self,
        identity: &RelationalBranchIdentity,
        control: &crate::mvcc::RelationalOperationControl,
    ) -> Result<
        (
            RelationalBranchBasisDescriptor,
            AdmittedRelationalBranchBasis,
        ),
        RelationalBranchBasisDenial,
    > {
        require_local_branch_identity(self, identity)?;
        let (descriptor, root, publication_cell, retention_binding) = {
            let cell = self
                .history
                .branch_cell(identity.branch_id())
                .ok_or_else(|| {
                    RelationalBranchBasisDenial::UnknownBranch(identity.branch_id().clone())
                })?;
            let snapshot = cell.atomic_snapshot();
            match snapshot.lifecycle_posture() {
                super::RelationalBranchLifecyclePosture::Live => {}
                super::RelationalBranchLifecyclePosture::Archived => {
                    return Err(RelationalBranchBasisDenial::ArchivedBranch(
                        identity.branch_id().clone(),
                    ));
                }
                super::RelationalBranchLifecyclePosture::Deleting => {
                    return Err(RelationalBranchBasisDenial::DeletingBranch(
                        identity.branch_id().clone(),
                    ));
                }
            }
            let root = snapshot.root().unwrap_or_else(|| {
                RelationalBranchRoot::empty_with_schema(
                    &self.config.schema.registry,
                    crate::schema::data::runtime_descriptor_semantics_policy()
                        .current_write_version(),
                )
            });
            let descriptor = descriptor_for_cell(&snapshot, &root)?;
            let retention_binding = cell
                .head_retention()
                .binding()
                .map_err(map_retention_denial)?;
            (descriptor, root, cell.publication_cell(), retention_binding)
        };
        require_not_interrupted(control, &retention_binding)?;
        let basis = issue_admitted_relational_branch_basis_with_retention(
            descriptor.clone(),
            identity.clone(),
            root,
            publication_cell,
            &retention_binding,
        )?;
        require_not_interrupted(control, &retention_binding)?;
        let basis = self
            .history
            .branch_cell(identity.branch_id())
            .expect("observed branch remains registered")
            .register_basis(basis)?;
        self.services.instrumentation.count_basis(|counters| {
            counters.basis_observations = counters.basis_observations.saturating_add(1);
        });
        Ok((descriptor, basis))
    }
}

fn require_not_interrupted(
    control: &crate::mvcc::RelationalOperationControl,
    retention_binding: &crate::history::retention::RelationalBranchRetentionBinding,
) -> Result<(), RelationalBranchBasisDenial> {
    match control.observe(crate::mvcc::RelationalInterruptionBoundary::ObservationAdmission) {
        None => Ok(()),
        Some(event)
            if event.interruption() == crate::mvcc::RelationalOperationInterruption::Cancelled =>
        {
            retention_binding.record_interruption(event);
            Err(RelationalBranchBasisDenial::Cancelled)
        }
        Some(event) => {
            retention_binding.record_interruption(event);
            Err(RelationalBranchBasisDenial::TimedOut)
        }
    }
}

fn map_retention_denial(
    denial: crate::history::retention::RelationalRetentionAcquisitionDenial,
) -> RelationalBranchBasisDenial {
    match denial {
        crate::history::retention::RelationalRetentionAcquisitionDenial::CapacityExhausted => {
            RelationalBranchBasisDenial::RetentionCapacityExhausted
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::OwnerUnavailable => {
            RelationalBranchBasisDenial::UnavailableRetainedTarget
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::IdentityExhausted => {
            RelationalBranchBasisDenial::RetentionIdentityExhausted
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::RootSetTooLarge => {
            RelationalBranchBasisDenial::OwnerFailure
        }
    }
}

pub(crate) fn descriptor_for_cell(
    cell: &super::RelationalBranchReferenceCell,
    root: &Arc<RelationalBranchRoot>,
) -> Result<RelationalBranchBasisDescriptor, RelationalBranchBasisDenial> {
    let visibility_commitment = root
        .axes()
        .map(|axes| axes.visibility.digest())
        .unwrap_or([0; 32]);
    let schema_commitment = root.schema_authority().authority_digest();
    require_root_matches_reference(&cell.observation(), root)?;
    let posture = match cell.lifecycle_posture() {
        super::RelationalBranchLifecyclePosture::Live => RelationalBranchBasisPosture::Live,
        super::RelationalBranchLifecyclePosture::Archived => RelationalBranchBasisPosture::Archived,
        super::RelationalBranchLifecyclePosture::Deleting => RelationalBranchBasisPosture::Deleting,
    };
    Ok(RelationalBranchBasisDescriptor::with_posture(
        super::basis::RelationalLiveBranchBasisDescriptorAxes {
            runtime_instance_id: cell.identity().runtime_instance_id(),
            branch_id: cell.identity().branch_id().clone(),
            reference: cell.observation().clone(),
            truth_version: cell.truth_version(),
            root_identity: root.id(),
            schema_commitment,
            visibility_commitment,
        },
        posture,
    ))
}
