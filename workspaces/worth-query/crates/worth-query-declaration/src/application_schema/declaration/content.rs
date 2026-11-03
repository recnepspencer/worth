use super::super::{
    ApplicationContentBinding, ApplicationContentBindingRecipe, ApplicationContentFieldRef,
};
use crate::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationEntityMarkerIdentity, ApplicationEntityRef,
    ApplicationFieldMarkerIdentity, ApplicationFieldUnit, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaMember, EqualityPosture, WritePosture,
};
use worth_foundational::facade::ContentRefId;

impl<S> ApplicationSchemaDeclarationBuilder<S> {
    /// Register a format once. Multiple fields reference this same declaration.
    pub fn content_format<C: ApplicationContentBinding>(mut self) -> Self {
        let recipe = ApplicationContentBindingRecipe::of::<C>();
        self.members.push(ApplicationSchemaMember::ContentFormat {
            declaration: recipe.declaration().clone(),
        });
        self.member_provenance.register_content_binding(recipe);
        self
    }
    pub fn content_field<E, A, F, W, Q, U, C>(
        mut self,
        entity: ApplicationEntityRef<S, E>,
        field: ApplicationContentFieldRef<S, E, A, F, W, Q, U, C>,
    ) -> Self
    where
        E: ApplicationEntityMarkerIdentity<S>,
        A: ApplicationAspectMarkerIdentity<S, E>,
        F: ApplicationFieldMarkerIdentity<
            S,
            E,
            A,
            Value = ContentRefId,
            Binding = crate::application_schema::ContentRefApplicationValueBinding,
        >,
        W: WritePosture,
        Q: EqualityPosture,
        U: ApplicationFieldUnit,
        C: ApplicationContentBinding,
    {
        let reference = field.field();
        self.member_provenance.register_content_field(
            crate::application_schema::ApplicationFieldBindingLocus::new(
                reference.entity(),
                reference.aspect(),
                reference.field(),
            ),
            ApplicationContentBindingRecipe::of::<C>(),
        );
        self.push_field(entity, reference, false, Some(*C::IDENTITY.bytes()))
    }
}
