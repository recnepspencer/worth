use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ConsumedCompositePublication, ProductBranchIdentity,
};

pub struct WorthQueryPerformedBranchAdoption {
    publication: ConsumedCompositePublication,
    source: ApplicationProgramRevision,
    target: ApplicationProgramRevision,
    selected_entity_count: usize,
}

impl WorthQueryPerformedBranchAdoption {
    pub(super) fn new(
        publication: ConsumedCompositePublication,
        source: ApplicationProgramRevision,
        target: ApplicationProgramRevision,
        selected_entity_count: usize,
    ) -> Self {
        Self {
            publication,
            source,
            target,
            selected_entity_count,
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

    pub fn product_branch_identity(&self) -> &ProductBranchIdentity {
        self.publication.new_product_head().branch_identity()
    }

    pub fn product_commit(&self) -> &CompositeCommitIdentity {
        self.publication.commit().identity()
    }
}
