//! One closure for every retained native-query consumer in an operation body.
use super::ApplicationOperationBodyProgram;
use crate::{
    application_query::ErasedApplicationQueryDefinition,
    application_schema::{
        ApplicationOperationBodyExternalPayloadSource as PayloadSource,
        ApplicationOperationBodyRequirementSource as RequirementSource,
    },
};
impl ApplicationOperationBodyProgram {
    pub fn values(&self) -> &[crate::application_schema::ApplicationOperationBodyValueProgram] {
        &self.values
    }
    pub fn with_untrusted_values(
        mut self,
        values: Vec<crate::application_schema::ApplicationOperationBodyValueProgram>,
    ) -> Self {
        self.values = values;
        self
    }
    pub fn observation_operands(
        &self,
    ) -> impl Iterator<Item = &crate::application_schema::ApplicationOperationBodyRequirementOperand>
    {
        self.observation_requirements()
            .flat_map(|rule| rule.operands())
            .chain(self.values.iter().flat_map(|value| value.operands()))
    }
    pub fn requires_query_context(&self) -> bool {
        self.query_contexts().next().is_some()
    }
    /// Includes the payload source even when no requirement or guard mentions it.
    pub fn query_contexts(
        &self,
    ) -> impl Iterator<Item = (&str, &ErasedApplicationQueryDefinition)> {
        self.observation_operands()
            .filter_map(|operand| match operand.source() {
                RequirementSource::QueryContext {
                    mutation_binding,
                    query,
                } => Some((mutation_binding.as_str(), query.as_ref())),
                _ => None,
            })
            .chain(
                self.external_emission
                    .iter()
                    .filter_map(|emission| match &emission.source {
                        PayloadSource::QueryContext {
                            mutation_binding,
                            query,
                        }
                        | PayloadSource::Evaluation {
                            mutation_binding,
                            query,
                        } => Some((mutation_binding.as_str(), query.as_ref())),
                        _ => None,
                    }),
            )
    }
}
