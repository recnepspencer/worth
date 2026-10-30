//! Managed authoring owns its selected parameter and actual codec provenance together.
use super::*;
use crate::application_query::{
    ApplicationManagedIndexBinding, ApplicationManagedIndexRef,
    ApplicationManagedIndexSelectionRecipe, ApplicationQueryManagedIndexMeaning,
};
use crate::application_schema::ApplicationSchema;

impl<S, Q: 'static, P, R, Scope> ApplicationQueryDefinitionBuilder<S, Q, P, R, Scope> {
    /// Declares the exact probe parameter and selects committed candidates from
    /// the index. A same-name explicit parameter is a duplicate declaration.
    pub fn managed_index<I: ApplicationManagedIndexBinding<S>, Parameter>(
        mut self,
        index: ApplicationManagedIndexRef<S, I>,
        parameter: ApplicationQueryParameterRef<Q, Parameter, I::ProbeBinding>,
    ) -> Self
    where
        S: ApplicationSchema,
    {
        self.definition
            .parameters
            .push(ApplicationQueryParameterDefinition::typed(parameter));
        let selection = ApplicationQueryManagedIndexMeaning::typed(index, parameter);
        self.definition.managed_index_recipe =
            Some(ApplicationManagedIndexSelectionRecipe::typed::<S, I, Q>(
                self.definition.name,
                self.definition.query_type.clone(),
                selection.clone(),
            ));
        self.definition.managed_index = Some(selection);
        self
    }
}
