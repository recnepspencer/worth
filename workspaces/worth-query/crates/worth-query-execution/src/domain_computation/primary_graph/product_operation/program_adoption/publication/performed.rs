use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ConsumedCompositePublication, ProductBranchIdentity,
};

/// A published branch adoption: the product branch now runs the target program.
///
/// Carries the source and target programs, how many entities were selected for
/// revalidation, any migration, the custody dispositions, and the product
/// commit.
pub struct WorthQueryPerformedBranchAdoption {
    publication: AdoptionPublication,
    source: ApplicationProgramRevision,
    target: ApplicationProgramRevision,
    selected_entity_count: usize,
    migration: Option<super::super::preparation::WorthQueryProgramMigrationDescription>,
    custody: super::super::preparation::WorthQueryProgramCustodyDispositionInventory,
}

enum AdoptionPublication {
    Live(ConsumedCompositePublication),
    Recovered {
        commit: CompositeCommitIdentity,
        branch: ProductBranchIdentity,
    },
}
impl WorthQueryPerformedBranchAdoption {
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn new(
        publication: ConsumedCompositePublication,
        source: ApplicationProgramRevision,
        target: ApplicationProgramRevision,
        selected_entity_count: usize,
        migration: Option<super::super::preparation::WorthQueryProgramMigrationDescription>,
        custody: super::super::preparation::WorthQueryProgramCustodyDispositionInventory,
    ) -> Self {
        Self {
            publication: AdoptionPublication::Live(publication),
            source,
            target,
            selected_entity_count,
            migration,
            custody,
        }
    }

    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn recovered(
        commit: CompositeCommitIdentity,
        branch: ProductBranchIdentity,
        source: ApplicationProgramRevision,
        target: ApplicationProgramRevision,
        selected_entity_count: usize,
        migration: Option<super::super::preparation::WorthQueryProgramMigrationDescription>,
        custody: super::super::preparation::WorthQueryProgramCustodyDispositionInventory,
    ) -> Self {
        Self {
            publication: AdoptionPublication::Recovered { commit, branch },
            source,
            target,
            selected_entity_count,
            migration,
            custody,
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

    pub fn product_branch_identity(&self) -> &ProductBranchIdentity {
        match &self.publication {
            AdoptionPublication::Live(publication) => {
                publication.new_product_head().branch_identity()
            }
            AdoptionPublication::Recovered { branch, .. } => branch,
        }
    }

    pub fn product_commit(&self) -> &CompositeCommitIdentity {
        match &self.publication {
            AdoptionPublication::Live(publication) => publication.commit().identity(),
            AdoptionPublication::Recovered { commit, .. } => commit,
        }
    }

    /// Exact Signal publication retained or produced by this adopted target.
    /// Descriptive component identity grants no permission to publish.
    pub fn signal_publication(
        &self,
    ) -> Option<&worth_runtime_world::facade::CompositeSignalPublicationIdentity> {
        match &self.publication {
            AdoptionPublication::Live(publication) => {
                publication.commit().signal_publication_identity()
            }
            AdoptionPublication::Recovered { .. } => None,
        }
    }

    /// Historical live work observations are not reconstructed after restart.
    pub fn relational_owner_contacts(&self) -> Option<u64> {
        match &self.publication {
            AdoptionPublication::Live(publication) => {
                Some(publication.cost_counters().relational_owner_contacts())
            }
            AdoptionPublication::Recovered { .. } => None,
        }
    }
}
