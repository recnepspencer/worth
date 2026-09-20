use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ConsumedCompositePublication, ProductBranchIdentity,
};

pub struct WorthQueryPerformedBranchAdoption {
    publication: ConsumedCompositePublication,
    source: ApplicationProgramRevision,
    target: ApplicationProgramRevision,
    selected_entity_count: usize,
    migration: Option<super::super::preparation::WorthQueryProgramMigrationDescription>,
}

impl WorthQueryPerformedBranchAdoption {
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn new(
        publication: ConsumedCompositePublication,
        source: ApplicationProgramRevision,
        target: ApplicationProgramRevision,
        selected_entity_count: usize,
        migration: Option<super::super::preparation::WorthQueryProgramMigrationDescription>,
    ) -> Self {
        Self {
            publication,
            source,
            target,
            selected_entity_count,
            migration,
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

    pub fn product_branch_identity(&self) -> &ProductBranchIdentity {
        self.publication.new_product_head().branch_identity()
    }

    pub fn product_commit(&self) -> &CompositeCommitIdentity {
        self.publication.commit().identity()
    }

    pub fn relational_owner_contacts(&self) -> u64 {
        self.publication.cost_counters().relational_owner_contacts()
    }
}
