//! The sealed program owner categories and their declared memberships.
use super::*;

impl<Schema> WorthQuerySelectedProgramOwner<'_, Schema> {
    /// Whether the selected program declares one output graph as a root.
    pub(in crate::domain_computation::primary_graph) fn owns_output_root(
        &self,
        root: TypeId,
    ) -> bool {
        self.root_graph_types.contains(&root)
    }
}

impl<Schema> sealed::WorthQueryProgramOwnership for WorthQuerySelectedProgramOwner<'_, Schema> {}

impl<Schema> WorthQueryProgramOwner<Schema> for WorthQuerySelectedProgramOwner<'_, Schema>
where
    Schema: ApplicationSchema,
{
    fn owned_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime
    }

    fn owned_revision(&self) -> &ApplicationProgramRevision {
        self.revision
    }

    fn owns_action(&self, binding: TypeId) -> bool {
        self.action_bindings.contains(&binding)
    }

    fn owns_output_source(&self, binding: TypeId) -> bool {
        self.output_source_bindings.contains(&binding)
    }
}

impl<Schema, Program> sealed::WorthQueryProgramOwnership
    for WorthQueryProgramApplicationRuntime<Schema, Program>
{
}

impl<Schema, Program> WorthQueryProgramOwner<Schema>
    for WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    fn owned_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime()
    }

    fn owned_revision(&self) -> &ApplicationProgramRevision {
        self.installed_program().revision()
    }

    fn owns_action(&self, binding: TypeId) -> bool {
        self.declares_action_binding(binding)
    }

    fn owns_output_source(&self, binding: TypeId) -> bool {
        self.requires_output_source(binding)
    }
}

impl<Schema, Spec, Program> sealed::WorthQueryProgramOwnership
    for WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
}

impl<Schema, Spec, Program> WorthQueryProgramOwner<Schema>
    for WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program: ApplicationProgramDefinition<Schema>,
{
    fn owned_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.program_runtime().owned_runtime()
    }

    fn owned_revision(&self) -> &ApplicationProgramRevision {
        self.program_runtime().owned_revision()
    }

    fn owns_action(&self, binding: TypeId) -> bool {
        self.program_runtime().owns_action(binding)
    }

    fn owns_output_source(&self, binding: TypeId) -> bool {
        self.program_runtime().owns_output_source(binding)
    }
}

impl<Schema, Program> sealed::WorthQueryProgramOwnership
    for WorthQuerySupportedProgramHandle<'_, Schema, Program>
{
}

impl<Schema, Program> WorthQueryProgramOwner<Schema>
    for WorthQuerySupportedProgramHandle<'_, Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    fn owned_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime()
    }

    fn owned_revision(&self) -> &ApplicationProgramRevision {
        self.record().revision()
    }

    fn owns_action(&self, binding: TypeId) -> bool {
        self.record().declares_action(binding)
    }

    fn owns_output_source(&self, binding: TypeId) -> bool {
        self.record().requires_output_source(binding)
    }
}
