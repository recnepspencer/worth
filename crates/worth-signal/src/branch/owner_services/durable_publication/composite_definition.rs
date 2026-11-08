//! A conditional successor uses the same authorized original preparation and driver.
use super::composite::CompositePublicationAuthorization;
use super::{
    SignalDurableAdvanceDenial, SignalDurableAdvanceTask, SignalDurableBranchMutationPort,
};
use crate::data::{
    aspect::SignalAspectLoweringOwner,
    conditional_execution::{
        InstalledSignalConditionalContract, SignalConditionalContractDefinition,
    },
    error::SignalError,
};
use crate::durability::{
    SignalCompositeAdvanceBinding, SignalDurableConditionalDefinitionChange,
    SignalDurableDefinitionPublication,
};
use crate::logic::transaction::SignalTransaction;
use worth_execution::ExecutionRequest;

impl<D, I, E, Ctx, T> SignalDurableBranchMutationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn begin_composite_definition_publication<F>(
        &self,
        execution: ExecutionRequest<'_, '_>,
        binding: SignalCompositeAdvanceBinding,
        grant: worth_proof::AdmittedCompositeSignalPublication,
        runtime_ctx: &mut Ctx,
        claimant: SignalAspectLoweringOwner,
        predecessor: InstalledSignalConditionalContract,
        definition: SignalConditionalContractDefinition,
        publication: Box<dyn SignalDurableDefinitionPublication>,
        apply: F,
    ) -> Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial>
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        let (expected, association, commitment) = binding.into_parts();
        self.begin_prepared(
            execution,
            &expected,
            runtime_ctx,
            apply,
            Some(Box::new(SignalDurableConditionalDefinitionChange {
                claimant,
                predecessor,
                definition,
                publication,
            })),
            Some(CompositePublicationAuthorization {
                grant,
                association,
                commitment,
            }),
        )
    }
}
