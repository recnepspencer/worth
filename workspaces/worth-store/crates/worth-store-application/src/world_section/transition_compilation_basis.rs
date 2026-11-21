//! The root a transition's successor contracts are compiled against.
use super::readmission::WorldReadmittedApplicationHome;
use super::ApplicationHomeTransitionPlanningRequestDenial as Denial;
use crate::open::readmission::transition::HomeTransitionPlanningOrigin;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis, durability::RelationalHomeProgramPlanningRoot,
};
use worth_runtime_world::facade::ProductBranchObservation;

/// Store's choice of compilation root. A resumed plan's root is readmitted
/// once: the successor compiles against it, then catalog readmission takes
/// the same root, so the compile basis and the author's lens cannot diverge.
pub struct ApplicationHomeTransitionCompilationBasis {
    origin: CompilationOrigin,
}
enum CompilationOrigin {
    Fresh(AdmittedRelationalBranchBasis),
    Resume(RelationalHomeProgramPlanningRoot),
}
impl ApplicationHomeTransitionCompilationBasis {
    pub fn basis(&self) -> &AdmittedRelationalBranchBasis {
        match &self.origin {
            CompilationOrigin::Fresh(basis) => basis,
            CompilationOrigin::Resume(root) => root.basis(),
        }
    }
    pub(super) fn planning_root(&self) -> Option<&RelationalHomeProgramPlanningRoot> {
        match &self.origin {
            CompilationOrigin::Fresh(_) => None,
            CompilationOrigin::Resume(root) => Some(root),
        }
    }
}

impl<D, I, E, Ctx, T> WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// A fresh plan compiles its successor on the current head. A recorded
    /// pending intent recompiles on its original planning predecessor, which
    /// Native readmits exactly from this recovery; a head already past the
    /// target step is never the compilation source.
    pub fn admit_home_program_transition_compilation_basis(
        &self,
        expected: &ProductBranchObservation,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeTransitionCompilationBasis, Denial> {
        request.consult().map_err(Denial::Execution)?;
        let context = &self.home.context;
        let origin = match context
            .transition
            .planning_origin(&context.descriptor.binding)
            .map_err(Denial::Transition)?
        {
            HomeTransitionPlanningOrigin::Fresh => {
                CompilationOrigin::Fresh(expected.basis().relational_basis().clone())
            }
            HomeTransitionPlanningOrigin::Resume => CompilationOrigin::Resume(
                self.home
                    .relational
                    .world_recovery_authority()
                    .ok_or(Denial::NativeRecoveryRequired)?
                    .readmit_home_program_planning_root(request, self.home.relational.runtime())
                    .map_err(|cause| Denial::PlanningRoot(Box::new(cause)))?,
            ),
        };
        Ok(ApplicationHomeTransitionCompilationBasis { origin })
    }
}
