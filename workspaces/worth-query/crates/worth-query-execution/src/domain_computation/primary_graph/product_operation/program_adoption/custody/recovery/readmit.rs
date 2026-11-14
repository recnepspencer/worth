//! Cold adoption custody rejoins one actual World catalogue original.
use super::*;
use worth_foundational::facade::DurableCompositePublicationAttemptIdentity;

#[derive(Debug)]
pub enum WorthQueryBranchAdoptionReadmissionDenial {
    World(RuntimeWorldRecoveryDenial),
    Obligation(crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial),
    Requirements(worth_query_installation::facade::WorthQueryProgramAdoptionRequirementsDenial),
    Custody(worth_query_installation::facade::WorthQueryProgramCustodyInventoryRequirement),
}
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// A fresh catalogue handle lends no principal or mutation authority. The
    /// ordinary recover_branch_adoption call still performs fresh admission.
    pub fn readmit_branch_adoption(
        &self,
        handle: &worth_runtime_world::facade::ProductUnpublishedRecoveryHandle,
    ) -> Result<WorthQueryBranchAdoptionRecovery, WorthQueryBranchAdoptionReadmissionDenial> {
        use crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial as Journal;
        use WorthQueryBranchAdoptionReadmissionDenial as Denial;
        let product = self
            .readmit_product_publication_recovery(handle)
            .map_err(Denial::World)?;
        let effects = product.inspect().map_err(Denial::World)?;
        let actual = effects.attempt_identity();
        let owner = actual
            .owner_identity()
            .durable()
            .ok_or(Denial::Obligation(Journal::ForeignHome))?;
        let attempt =
            DurableCompositePublicationAttemptIdentity::from_recorded(owner, actual.ordinal());
        let support = self
            .installed_program_support()
            .ok_or(Denial::Obligation(Journal::SupportUnavailable))?;
        let row = support
            .lifecycle()
            .branch_adoption(attempt)
            .ok_or(Denial::Obligation(Journal::UnknownObligation))?;
        let expected = effects.expected_product();
        if expected.selected_commit().owner_identity().durable() != Some(row.expected.owner())
            || expected.selected_commit().ordinal() != row.expected.ordinal()
            || expected.branch_identity().name().as_str() != row.branch
            || expected.lifecycle_incarnation().ordinal() != row.lifecycle
            || expected.reference_generation().get().checked_add(1) != Some(row.generation)
        {
            return Err(Denial::Obligation(Journal::ObligationSubjectMismatch));
        }
        let (source, target, support_custody) = support
            .lifecycle()
            .readmit_adoption_custody(&row)
            .map_err(Denial::Obligation)?;
        let requirements = support
            .adoption_requirements(self.installed_schema(), &source, &target)
            .map_err(Denial::Requirements)?;
        let custody = super::super::super::preparation::derive_custody_dispositions(&requirements)
            .map_err(Denial::Custody)?;
        let selected_entity_count = usize::try_from(row.selected_entity_count)
            .map_err(|_| Denial::Obligation(Journal::MalformedFrame))?;
        let migration=row.migration.map(|(operation,count)| {
            usize::try_from(count).map(|count|super::super::super::preparation::WorthQueryProgramMigrationDescription::from_recorded(operation,count))
                .map_err(|_|Denial::Obligation(Journal::MalformedFrame))
        }).transpose()?;
        drop(effects);
        Ok(WorthQueryBranchAdoptionRecovery::new(
            source,
            target,
            selected_entity_count,
            migration,
            custody,
            product,
            support_custody,
        ))
    }
}
