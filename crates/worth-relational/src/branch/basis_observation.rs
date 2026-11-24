use super::basis_observation_inputs::BranchObservationInputs;
use super::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisDenial, RelationalBranchBasisDescriptor,
    RelationalBranchIdentity,
};
use crate::runtime::RelationalRuntime;
use worth_relational_contracts::internal_port::branch::identity::RelationalBranchIdentityOwnerPort as _;

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
    ///     .build()
    ///     .expect("valid native runtime configuration");
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
        BranchObservationInputs::from(self).observe(identity, || {
            control.observe(crate::runtime::operation_control::interruption_event::RelationalInterruptionBoundary::ObservationAdmission)
        })
    }
}
