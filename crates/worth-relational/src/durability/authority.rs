mod append_authority;
mod authority_continuity;
mod checkpoint_capture;
mod checkpoint_image;
mod checkpointing;
mod diagnostics;
mod recovery;
mod runtime_rebuild;

use crate::branch::AdmittedRelationalBranchBasis;
use crate::history::data::BranchId;
use crate::runtime::RelationalRuntime;

pub(crate) use append_authority::DurableAppendAuthority;

pub struct DurabilityAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl<'runtime> DurabilityAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}

/// The exclusive recovery lane.
///
/// Recovery rebuilds a whole runtime from durable evidence and then installs it
/// in place of the live one, so it is the single durability operation that
/// cannot run against a shared borrow. It is a separate wrapper rather than a
/// method on the shared authority so the exclusive requirement is visible at
/// every call site instead of infecting ordinary durability work.
pub struct DurabilityRecoveryAuthority<'runtime> {
    runtime: &'runtime mut RelationalRuntime,
}

/// Linear evidence that this exact Relational runtime and its captured branch
/// images were rebuilt from a verified native checkpoint. The value is
/// owner-issued and deliberately non-serializable, so descriptive checkpoint
/// presence or later branch movement cannot impersonate successful recovery.
#[derive(Debug)]
pub struct RecoveredRelationalRuntimeAuthority {
    runtime_instance_id: u64,
    recovered_branch_images: std::collections::BTreeMap<
        BranchId,
        (
            crate::branch::RelationalBranchReferenceObservation,
            crate::branch::RelationalBranchVersion,
        ),
    >,
}

impl RecoveredRelationalRuntimeAuthority {
    pub fn admit_basis(
        self,
        basis: AdmittedRelationalBranchBasis,
    ) -> Result<RecoveredRelationalBranchBasis, AdmittedRelationalBranchBasis> {
        let descriptor = basis.descriptor();
        let matches_recovered_image = self
            .recovered_branch_images
            .get(descriptor.branch_id())
            .is_some_and(|(reference, truth_version)| {
                reference == descriptor.reference() && *truth_version == descriptor.truth_version()
            });
        if descriptor.runtime_instance_id() == self.runtime_instance_id && matches_recovered_image {
            Ok(RecoveredRelationalBranchBasis { basis })
        } else {
            Err(basis)
        }
    }
}

/// An operational branch basis proven to belong to a successfully recovered
/// Relational runtime. Only recovery authority can construct this wrapper.
#[derive(Debug)]
pub struct RecoveredRelationalBranchBasis {
    basis: AdmittedRelationalBranchBasis,
}

impl RecoveredRelationalBranchBasis {
    pub fn into_basis(self) -> AdmittedRelationalBranchBasis {
        self.basis
    }
}

impl<'runtime> DurabilityRecoveryAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime mut RelationalRuntime) -> Self {
        Self { runtime }
    }
}

impl RelationalRuntime {
    pub fn durability_authority(&self) -> DurabilityAuthority<'_> {
        DurabilityAuthority::new(self)
    }

    pub fn durability_recovery(&mut self) -> DurabilityRecoveryAuthority<'_> {
        DurabilityRecoveryAuthority::new(self)
    }
}
