//! A fresh created entity may point to admission's principal, never a caller-selected ID.
use super::*;
use crate::application_operation::ApplicationMutationBinding;
use crate::application_schema::{ApplicationFieldBindingLocus, ApplicationSchema};
impl<S: ApplicationSchema, O, I, E> ApplicationOperationBodyCreationBuilder<S, O, I, E> {
    /// Links this creation to the exact principal admitted for B. Required Link
    /// membership and the principal endpoint are compiler-visible; cold descriptions
    /// are independently checked before installation. No identity input is accepted.
    /// ```compile_fail,E0277
    /// use worth_query_declaration::facade::{application_schema::*,application_operation::ApplicationMutationBinding};
    /// fn missing_link<S:ApplicationSchema,O,I,E,B,R>(c:ApplicationOperationBodyCreationBuilder<S,O,I,E>,r:ApplicationRelationRef<S,R,E,B::Principal>)
    /// where B:ApplicationMutationBinding<S,Operation=O,Input=I> {
    ///     let _=c.link_to_principal::<B,R>(r,"creator");
    /// }
    /// ```
    /// ```compile_fail,E0308
    /// use worth_query_declaration::facade::{application_schema::*,application_operation::ApplicationMutationBinding};
    /// fn wrong_endpoint<S:ApplicationSchema,O,I,E,Other,B,R>(c:ApplicationOperationBodyCreationBuilder<S,O,I,E>,r:ApplicationRelationRef<S,R,E,Other>)
    /// where B:ApplicationMutationBinding<S,Operation=O,Input=I>,R:OperationLinks<O> {
    ///     let _=c.link_to_principal::<B,R>(r,"creator");
    /// }
    /// ```
    /// ```no_run
    /// use worth_query_declaration::facade::{application_schema::*,application_operation::ApplicationMutationBinding};
    /// fn declared<S:ApplicationSchema,O,I,E,B,R>(c:ApplicationOperationBodyCreationBuilder<S,O,I,E>,r:ApplicationRelationRef<S,R,E,B::Principal>)
    /// where B:ApplicationMutationBinding<S,Operation=O,Input=I>,R:OperationLinks<O> {
    ///     let _=c.link_to_principal::<B,R>(r,"creator");
    /// }
    /// ```
    pub fn link_to_principal<B, R>(
        mut self,
        relation: ApplicationRelationRef<S, R, E, B::Principal>,
        key: &str,
    ) -> Result<Self, Denial>
    where
        B: ApplicationMutationBinding<S, Operation = O, Input = I>,
        R: OperationLinks<O>,
    {
        let principal = B::principal_binding();
        self = self.link(
            relation.name(),
            relation.from(),
            relation.to(),
            key,
            ApplicationOperationBodyCreationLinkDirection::ToScope,
        )?;
        self.links.last_mut().expect("new exact link").source =
            ApplicationOperationBodyCreationLinkSource::Principal {
                mutation_binding: B::IDENTITY.into(),
                principal_binding: principal.name().into(),
                identity: ApplicationFieldBindingLocus::new(
                    principal.principal_entity(),
                    principal.principal_identity_aspect(),
                    principal.principal_identity_field(),
                ),
            };
        Ok(self)
    }
}
