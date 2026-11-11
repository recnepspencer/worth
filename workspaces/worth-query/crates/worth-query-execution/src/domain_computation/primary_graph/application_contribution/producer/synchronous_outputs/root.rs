//! Fixed and discovered required roots seed the same candidate wave.
use super::*;
pub(super) trait Discovered<S: ApplicationSchema>: Send + 'static {
    fn order(
        &self,
    ) -> &worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder;
    fn seed(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        wave: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
}
pub(super) enum Root<S: ApplicationSchema> {
    Fixed(Box<dyn capture::RequiredTree<S>>),
    Discovered(Box<dyn Discovered<S>>),
}
impl<S: ApplicationSchema> Root<S> {
    /// Actual allocated container layouts, obtained from their trait-object vtables.
    pub(super) fn container_bytes(&self) -> usize {
        std::mem::size_of_val(self)
            + match self {
                Self::Fixed(tree) => std::mem::size_of_val(tree.as_ref()),
                Self::Discovered(tree) => std::mem::size_of_val(tree.as_ref()),
            }
    }

    pub(super) fn order(
        &self,
    ) -> &worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder
    {
        match self {
            Self::Fixed(tree) => tree.order(),
            Self::Discovered(tree) => tree.order(),
        }
    }
    pub(super) fn seed(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        wave: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        match self {
            Self::Fixed(tree) => tree.seed(context, wave),
            Self::Discovered(tree) => tree.seed(context, candidate, wave),
        }
    }
}
