use super::WorthQueryRuntimeBuilder;

impl WorthQueryRuntimeBuilder {
    /// Installs one Query-owned request/response source into the conditional
    /// graph before the runtime is sealed.
    pub fn owned_bridge_async_declaration(
        mut self,
        declaration: super::super::WorthQueryOwnedAsyncRequestDeclaration,
    ) -> Self {
        self.pending_owned_async_declarations.push(declaration);
        self
    }
}
