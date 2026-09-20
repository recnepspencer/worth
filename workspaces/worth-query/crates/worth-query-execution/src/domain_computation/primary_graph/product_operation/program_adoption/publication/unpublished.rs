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
    product: crate::domain_computation::WorthQueryProductUnpublishedApplication,
}

impl WorthQueryUnpublishedBranchAdoption {
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn new(
        source: ApplicationProgramRevision,
        target: ApplicationProgramRevision,
        selected_entity_count: usize,
        migration: Option<super::super::preparation::WorthQueryProgramMigrationDescription>,
        owner_effects: ProductUnpublishedOwnerEffects,
        recovery: RuntimeWorldRecoveryPort,
        disposition: crate::domain_computation::primary_graph::WorthQueryUnpublishedIdempotencyDisposition,
    ) -> Self {
        Self {
            source,
            target,
            selected_entity_count,
            migration,
            product: crate::domain_computation::WorthQueryProductUnpublishedApplication::new(
                owner_effects,
                recovery,
                disposition,
            ),
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
            self.product.into_recovery(),
        )
    }
}
