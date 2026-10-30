//! Immutable installed provider purpose for one exact allowed Parent.
use super::*;
use crate::{
    application_query::{
        WorthQueryInstalledApplicationQueryIdentity, WorthQueryInstalledGraphReadContract,
    },
    graph_obligation::{
        bind_managed_index_obligations, WorthQueryInstalledGraphObligationInspection,
        WorthQueryInstalledGraphObligationSet,
    },
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

/// Descriptive installed meaning; execution still requires its actual leaf and session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledManagedIndexReadMeaning {
    schema: ApplicationSchemaBindingIdentity,
    definition: Arc<ErasedApplicationManagedIndexDefinition>,
    source_binding: String,
    source_query: WorthQueryInstalledApplicationQueryIdentity,
    source_graph: WorthQueryInstalledGraphReadContract,
    parent_binding: String,
    parent_query: WorthQueryInstalledApplicationQueryIdentity,
    parent_graph: WorthQueryInstalledGraphReadContract,
    maximum_source_results: usize,
}

/// One cold compiled correspondence, shared by all live associations for this Parent.
#[derive(Debug)]
pub struct WorthQueryInstalledManagedIndexAccessContract {
    source_obligation: crate::graph_obligation::WorthQueryInstalledGraphObligationSetIdentity,
    parent_obligation: crate::graph_obligation::WorthQueryInstalledGraphObligationSetIdentity,
    meaning: Arc<WorthQueryInstalledManagedIndexReadMeaning>,
    obligations: Arc<WorthQueryInstalledGraphObligationSet>,
}

impl WorthQueryInstalledManagedIndexAccessContract {
    pub(super) fn compile(
        schema: ApplicationSchemaBindingIdentity,
        recipe: &ApplicationManagedIndexBindingDescriptor,
        source: &Binding,
        parent: &Binding,
    ) -> Result<Self, Denial> {
        let meaning = Arc::new(WorthQueryInstalledManagedIndexReadMeaning {
            schema,
            definition: Arc::new(recipe.meaning().clone()),
            source_binding: source.descriptor().identity().to_owned(),
            source_query: source.query_identity().clone(),
            source_graph: source.query().read_graph().clone(),
            parent_binding: parent.descriptor().identity().to_owned(),
            parent_query: parent.query_identity().clone(),
            parent_graph: parent.query().read_graph().clone(),
            maximum_source_results: source.limits().maximum_results().get(),
        });
        let obligations = bind_managed_index_obligations(&meaning).map_err(|cause| {
            super::super::binding::graph_obligation_denial(&recipe.meaning().parts().name, cause)
        })?;
        Ok(Self {
            source_obligation: source.query().graph_obligations().identity().clone(),
            parent_obligation: parent.query().graph_obligations().identity().clone(),
            meaning,
            obligations: Arc::new(obligations),
        })
    }
    /// The genuine compiled Source Query obligation, distinct from this provider purpose.
    pub fn source_obligation(
        &self,
    ) -> &crate::graph_obligation::WorthQueryInstalledGraphObligationSetIdentity {
        &self.source_obligation
    }
    /// The genuine compiled Parent Query obligation that must remain admitted.
    pub fn parent_obligation(
        &self,
    ) -> &crate::graph_obligation::WorthQueryInstalledGraphObligationSetIdentity {
        &self.parent_obligation
    }
    pub fn meaning(&self) -> &WorthQueryInstalledManagedIndexReadMeaning {
        &self.meaning
    }
    pub fn graph_obligations(&self) -> WorthQueryInstalledGraphObligationInspection<'_> {
        self.obligations.inspect()
    }
    #[doc(hidden)]
    pub fn retain_graph_obligations_for_admission(
        &self,
    ) -> Arc<WorthQueryInstalledGraphObligationSet> {
        Arc::clone(&self.obligations)
    }
}

impl WorthQueryInstalledManagedIndexReadMeaning {
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }
    pub fn definition(&self) -> &ErasedApplicationManagedIndexDefinition {
        &self.definition
    }
    pub fn source_binding(&self) -> &str {
        &self.source_binding
    }
    pub fn source_query(&self) -> &WorthQueryInstalledApplicationQueryIdentity {
        &self.source_query
    }
    pub fn source_graph(&self) -> &WorthQueryInstalledGraphReadContract {
        &self.source_graph
    }
    pub fn parent_binding(&self) -> &str {
        &self.parent_binding
    }
    pub fn parent_query(&self) -> &WorthQueryInstalledApplicationQueryIdentity {
        &self.parent_query
    }
    pub fn parent_graph(&self) -> &WorthQueryInstalledGraphReadContract {
        &self.parent_graph
    }
    pub fn maximum_source_results(&self) -> usize {
        self.maximum_source_results
    }
}
