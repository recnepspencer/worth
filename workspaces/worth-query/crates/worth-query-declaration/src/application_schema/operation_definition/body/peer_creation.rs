//! Independent creation roots reuse typed initialization without inheriting disposal.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
    ApplicationOperationBodyCreationBuilder,
};
use crate::application_schema::{ApplicationEntityRef, OperationCreates};
impl<S, O, I, E> ApplicationOperationBodyCreationBuilder<S, O, I, E> {
    /// Declares an independent creation in this body's original admitted scope.
    /// Unlike a child, a peer does not inherit its sibling's disposal or reuse.
    ///
    /// ```compile_fail,E0277
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn missing_create<S,O,I,E,P>(creation:ApplicationOperationBodyCreationBuilder<S,O,I,E>,peer:ApplicationEntityRef<S,P>) {
    ///     let _=creation.with_peer(peer,"peer",Ok);
    /// }
    /// ```
    ///
    /// ```no_run
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn declared_create<S,O,I,E,P:OperationCreates<O>>(creation:ApplicationOperationBodyCreationBuilder<S,O,I,E>,peer:ApplicationEntityRef<S,P>) {
    ///     let _=creation.with_peer(peer,"peer",Ok);
    /// }
    /// ```
    pub fn with_peer<P>(
        mut self,
        entity: ApplicationEntityRef<S, P>,
        key: &str,
        build: impl FnOnce(
            ApplicationOperationBodyCreationBuilder<S, O, I, P>,
        )
            -> Result<ApplicationOperationBodyCreationBuilder<S, O, I, P>, Denial>,
    ) -> Result<Self, Denial>
    where
        P: OperationCreates<O>,
    {
        let scope = self
            .parent
            .scope
            .as_deref()
            .or_else(|| self.parent.steps.first().map(|s| s.field().entity()))
            .ok_or(Denial::Empty)?;
        let peer = build(
            ApplicationOperationBodyBuilder::empty_in_scope(scope).create_in_scope(entity, key)?,
        )?
        .finish()?;
        // A peer callback authors a creation site, not an independent operation.
        // No parent state may disappear when merging it into the one body.
        if !peer.program.scope_fields().is_empty()
            || !peer.program.requirements().is_empty()
            || !peer.program.outputs().is_empty()
            || peer.program.succession().is_some()
            || peer.program.external_emission().is_some()
            || !peer.program.retirements().is_empty()
            || !peer.program.path_input_expectations().is_empty()
            || !peer.program.path_value_assignments().is_empty()
            || !peer.program.reference_inputs().is_empty()
            || peer.external_emission_recipe.is_some()
        {
            return Err(Denial::InvalidCreationLink);
        }
        for value in peer.program.values() {
            self.parent.retain_value(value)?;
        }
        let creation = peer
            .program
            .creation()
            .expect("peer builder creates")
            .clone();
        self.peers.push(creation);
        Ok(self)
    }
}
