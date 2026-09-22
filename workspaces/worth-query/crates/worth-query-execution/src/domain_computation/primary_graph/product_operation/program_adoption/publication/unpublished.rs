use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_runtime_world::facade::{
    ProductBranchObservation, ProductUnpublishedCause, ProductUnpublishedNextAction,
    ProductUnpublishedOwnerEffects, RuntimeWorldRecoveryPort,
};

#[derive(Debug)]
pub struct WorthQueryUnpublishedBranchAdoption {
    source: ApplicationProgramRevision,
    target: ApplicationProgramRevision,
    selected_entity_count: usize,
    migration: Option<super::super::preparation::WorthQueryProgramMigrationDescription>,
    custody: super::super::preparation::WorthQueryProgramCustodyDispositionInventory,
    product: crate::domain_computation::WorthQueryProductUnpublishedApplication,
    support_custody:
        crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportCustody,
}

impl WorthQueryUnpublishedBranchAdoption {
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn new(
        source: ApplicationProgramRevision,
        target: ApplicationProgramRevision,
        selected_entity_count: usize,
        migration: Option<super::super::preparation::WorthQueryProgramMigrationDescription>,
        custody: super::super::preparation::WorthQueryProgramCustodyDispositionInventory,
        owner_effects: ProductUnpublishedOwnerEffects,
        recovery: RuntimeWorldRecoveryPort,
        disposition: crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition,
        support_custody: crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportCustody,
    ) -> Self {
        Self {
            source,
            target,
            selected_entity_count,
            migration,
            custody,
            product: crate::domain_computation::WorthQueryProductUnpublishedApplication::new(
                owner_effects,
                recovery,
                disposition,
            ),
            support_custody,
        }
    }

    pub fn source(&self) -> &ApplicationProgramRevision {
        &self.source
    }

    pub fn target(&self) -> &ApplicationProgramRevision {
        &self.target
    }

    pub const fn selected_entity_count(&self) -> usize {
        self.selected_entity_count
    }

    pub const fn migration(
        &self,
    ) -> Option<&super::super::preparation::WorthQueryProgramMigrationDescription> {
        self.migration.as_ref()
    }

    pub const fn custody(
        &self,
    ) -> &super::super::preparation::WorthQueryProgramCustodyDispositionInventory {
        &self.custody
    }

    pub fn cause(&self) -> ProductUnpublishedCause {
        self.product.cause()
    }

    pub fn expected_product(&self) -> &ProductBranchObservation {
        self.product.expected_product()
    }

    pub fn next_actions(&self) -> &[ProductUnpublishedNextAction] {
        self.product.next_actions()
    }

    pub fn relational_requires_settlement(&self) -> bool {
        self.product.relational_requires_settlement()
    }

    pub fn into_recovery(self) -> super::super::custody::WorthQueryBranchAdoptionRecovery {
        super::super::custody::WorthQueryBranchAdoptionRecovery::new(
            self.source,
            self.target,
            self.selected_entity_count,
            self.migration,
            self.custody,
            self.product.into_recovery(),
            self.support_custody,
        )
    }
}
