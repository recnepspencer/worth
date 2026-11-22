//! The ordinary handler entrance delegates the same scoped issued selection.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryInvariantProjectionComponent;
use worth_query_declaration::facade::application_schema::{
    ApplicationComponentSetRef, ApplicationOperationMarkerIdentity,
};
impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
where
    Binding::Operation: ApplicationOperationMarkerIdentity<Schema>,
{
    /// Read a whole component under this admitted operation's installed visibility.
    pub fn component_of<Set: OperationReads<Binding::Operation>, Entity>(
        &mut self,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryInvariantProjectionComponent<Schema, Set, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        self.component_execution(set.name())?;
        self.reader.component_of(set, member)
    }
    /// Enumerate complete components in the admitted root's complete selection.
    pub fn components<Set: OperationReads<Binding::Operation>, Entity>(
        &mut self,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
    ) -> Result<
        Vec<WorthQueryInvariantProjectionComponent<Schema, Set, Entity>>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        self.component_execution(set.name())?;
        self.reader.components(set)
    }
    /// Count the same complete selection and retain its negative/empty evidence.
    pub fn component_count<Set: OperationReads<Binding::Operation>, Entity>(
        &mut self,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
    ) -> Result<usize, WorthQueryInvariantProjectionTraversalDenial> {
        self.component_execution(set.name())?;
        self.reader.component_count(set)
    }
    fn component_execution(
        &self,
        subject: &str,
    ) -> Result<(), WorthQueryInvariantProjectionTraversalDenial> {
        self.execution().execution_request().map(|_|()).map_err(|cause|WorthQueryInvariantProjectionTraversalDenial::new(crate::domain_computation::primary_graph::WorthQueryInvariantProjectionTraversalDenialKind::ExecutionRequest(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(cause)),subject))
    }
}
