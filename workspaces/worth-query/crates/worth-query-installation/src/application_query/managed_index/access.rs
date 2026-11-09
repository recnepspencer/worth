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
    parent_disclosure:
        worth_query_declaration::facade::application_query::ApplicationQueryDisclosurePosture,
    maximum_source_results: usize,
    predicate_profile: Result<
        super::WorthQueryManagedIndexPredicateProfile,
        super::WorthQueryManagedIndexPredicateProfileDenial,
    >,
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
        budget: worth_foundational::facade::CanonicalDigestWorkBudget,
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
            parent_disclosure: parent.query().disclosure().posture(),
            maximum_source_results: source.limits().maximum_results().get(),
            predicate_profile: super::matching::compile(recipe, source, parent),
        });
        if meaning.match_posture() == worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture::Refined
            && meaning.predicate_profile.is_err() {
            return Err(super::denied(super::Kind::ManagedIndexParentUnsupported, &recipe.meaning().parts().name));
        }
        let obligations = bind_managed_index_obligations(budget, &meaning).map_err(|cause| {
            super::super::denial::graph_obligation_denial(&recipe.meaning().parts().name, cause)
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
    pub fn parent_disclosure_posture(
        &self,
    ) -> worth_query_declaration::facade::application_query::ApplicationQueryDisclosurePosture {
        self.parent_disclosure
    }
    pub fn match_posture(
        &self,
    ) -> worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture
    {
        self.parent_graph
            .managed_index_selection()
            .expect("managed access has its compiled selector")
            .posture()
    }
    /// The compiled tracked full-Parent profile; still requires a maintained
    /// Source, actual provider association, and fresh read admission at use.
    pub fn predicate_profile(
        &self,
    ) -> Result<
        super::WorthQueryManagedIndexPredicateProfile,
        super::WorthQueryManagedIndexPredicateProfileDenial,
    > {
        self.predicate_profile
    }

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
