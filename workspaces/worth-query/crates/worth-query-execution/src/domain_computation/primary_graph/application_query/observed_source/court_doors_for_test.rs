//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryObservedSourceFootprint {
    #[doc(hidden)]
    pub fn new_for_test(
        root: EntityId,
        complete: bool,
        entities: Vec<EntityId>,
        aspects: Vec<WorthQueryObservedFieldRevision>,
        root_selection: Option<std::sync::Arc<WorthQueryObservedRootSelection>>,
        adjacencies: Vec<WorthQueryObservedAdjacencyRevision>,
    ) -> Self {
        Self {
            root,
            complete,
            entities,
            aspects,
            root_selection,
            adjacencies,
        }
    }
}

impl WorthQueryObservedSourceFootprint {
    #[doc(hidden)]
    pub fn aspects_for_test(&self) -> &Vec<WorthQueryObservedFieldRevision> {
        &self.aspects
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn branch_for_test(&self) -> &std::sync::Arc<BranchId> {
        &self.branch
    }
}

impl WorthQueryObservedSourceFootprint {
    #[doc(hidden)]
    pub fn entities_for_test(&self) -> &Vec<EntityId> {
        &self.entities
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn model_root_for_test(&self) -> &EntityId {
        &self.model_root
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn parameters_for_test(&self) -> &std::sync::Arc<WorthQueryRetainedObservedParameters> {
        &self.parameters
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn query_identifier_for_test(&self) -> &std::sync::Arc<String> {
        &self.query_identifier
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn query_identity_for_test(&self) -> &WorthQueryInstalledApplicationQueryIdentity {
        &self.query_identity
    }
}

impl WorthQueryObservedSourceFootprint {
    #[doc(hidden)]
    pub fn root_for_test(&self) -> &EntityId {
        &self.root
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn runtime_authority_for_test(&self) -> &u64 {
        &self.runtime_authority
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn schema_binding_for_test(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn selection_for_test(&self) -> &WorthQueryApplicationBasisSelectionIdentity {
        &self.selection
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn source_meaning_for_test(
        &self,
    ) -> &std::sync::Arc<source_identity::WorthQueryObservedSourceMeaning> {
        &self.source_meaning
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn source_root_for_test(&self) -> EntityId {
        Self::source_root(self)
    }
}

impl<Query> WorthQueryObservedSource<Query> {
    #[doc(hidden)]
    pub fn validate_completeness_for_test(
        &self,
        subject: &str,
    ) -> Result<(), WorthQuerySourceExpectationDenial> {
        Self::validate_completeness(self, subject)
    }
}

impl WorthQueryObservedSourceFootprint {
    #[doc(hidden)]
    pub fn complete_for_test(&self) -> &bool {
        &self.complete
    }
}
