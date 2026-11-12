//! Admit the exact Native source selected by a World creation.
use super::*;
use worth_relational::facade::{
    branch::{
        AdmittedRelationalBranchBasis, AdmittedRelationalForkSourceBasis,
        RelationalBranchReferenceObservation, RelationalBranchVersion,
        RelationalForkSourceDescriptor,
    },
    history::BranchId,
};
impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn observe_creation_native_source(
        &self,
        expected: &ProductBranchObservation,
    ) -> Result<AdmittedRelationalForkSourceBasis, RuntimeWorldBranchAdmissionDenial> {
        let admitted = expected.basis().relational_basis();
        let (descriptor, source) = self
            .state
            .relational_services()
            .fork_port()
            .observe_fork_source(admitted.descriptor().branch_id())
            .map_err(|cause| super::native_fork_denial::admission_denial(&cause))?;
        if source_is_the_admitted_basis(&observed_axes(&descriptor), &admitted_axes(admitted)) {
            Ok(source)
        } else {
            Err(RuntimeWorldBranchAdmissionDenial::ForkSourceChanged)
        }
    }
}
/// The axes a fresh fork-source observation shares with the admitted source
/// basis, lifted off both owner types so the comparison is total and can be
/// proved one axis at a time without an owner.
///
/// Two of the four axes are live and two are structural. `observation` and
/// `truth_version` are live: the Relational owner moves them whenever the
/// source branch moves, and that drift is exactly what this comparison
/// exists to refuse. `source_branch` and `runtime_instance_id` are
/// structural: the fresh observation is requested *for* the admitted basis's
/// own branch id, on this process's own owner, so they can differ only if
/// the port answers about a branch it was never asked about. They are
/// compared anyway, and proved case by case, so a later port, transport, or
/// cross-instance change cannot make that mismatch silent.
struct ForkSourceAxes<'a> {
    runtime_instance_id: u64,
    source_branch: &'a BranchId,
    observation: &'a RelationalBranchReferenceObservation,
    truth_version: RelationalBranchVersion,
}

/// A single differing axis means the branch the owner would fork is not the
/// branch this creation was admitted against.
fn source_is_the_admitted_basis(
    observed: &ForkSourceAxes<'_>,
    admitted: &ForkSourceAxes<'_>,
) -> bool {
    observed.runtime_instance_id == admitted.runtime_instance_id
        && observed.source_branch == admitted.source_branch
        && observed.observation == admitted.observation
        && observed.truth_version == admitted.truth_version
}

/// Axes of the source observation the Relational owner just issued.
fn observed_axes(descriptor: &RelationalForkSourceDescriptor) -> ForkSourceAxes<'_> {
    ForkSourceAxes {
        runtime_instance_id: descriptor.runtime_instance_id(),
        source_branch: descriptor.source_branch(),
        observation: descriptor.observation(),
        truth_version: descriptor.truth_version(),
    }
}

/// Axes of the source basis this creation was admitted against.
fn admitted_axes(admitted: &AdmittedRelationalBranchBasis) -> ForkSourceAxes<'_> {
    let basis = admitted.descriptor();
    ForkSourceAxes {
        runtime_instance_id: basis.runtime_instance_id(),
        source_branch: basis.branch_id(),
        observation: basis.reference(),
        truth_version: basis.truth_version(),
    }
}

#[cfg(test)]
#[path = "creation/execution/forks/source_axes_tests.rs"]
mod source_axes_tests;
