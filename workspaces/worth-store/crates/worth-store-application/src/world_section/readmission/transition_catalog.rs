//! The real Home lends its committed original intent to Native readmission.
use super::WorldReadmittedApplicationHome;
use worth_execution::ExecutionRequest;
use worth_proof::AdmittedHomeProgramTransitionCatalog;
use worth_relational::facade::runtime::RelationalHomeProgramCatalogReadmissionSource;

impl<D, I, E, Ctx, T> WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// A borrow of the actual immutable intent cannot outlive its payer or
    /// become an execution permission, even after its World occurrence exists.
    pub fn admit_home_program_transition_catalog(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        AdmittedHomeProgramTransitionCatalog<RelationalHomeProgramCatalogReadmissionSource<'_>>,
        crate::ApplicationHomeTransitionIntentDenial,
    > {
        crate::open::readmission::transition::admit_catalog(&self.home.context, request)
    }
}
