use crate::facade::registry::descriptor::WorthUiQueryViewRegistration;

use super::{
    WorthUiApplicationBuilder, WorthUiProjectionRegistrationError,
    WorthUiQueryViewRegistrationError,
};

impl<ChangeProfileState, IntentWiringState>
    WorthUiApplicationBuilder<ChangeProfileState, IntentWiringState>
{
    /// Register an installed Query view as one coherent definition and
    /// runtime-affine authority. Query posture cannot be assembled piecemeal.
    pub fn register_query_view(
        mut self,
        registration: impl Into<WorthUiQueryViewRegistration>,
    ) -> Result<Self, WorthUiQueryViewRegistrationError> {
        let (view, visible_state_bindings, denial_presentation) = registration.into().into_parts();
        let definition = view.definition().clone();
        let id = crate::capability::ViewBindingId::new(definition.identity().as_str())
            .map_err(WorthUiQueryViewRegistrationError::InvalidIdentity)?;
        let family = match definition.shape() {
            worth_ui_query_binding::WorthUiQueryViewShape::Collection => {
                crate::capability::ViewBindingFamily::collection()
            }
            worth_ui_query_binding::WorthUiQueryViewShape::Detail => {
                crate::capability::ViewBindingFamily::detail()
            }
        };
        self.query_binding_plan = self
            .query_binding_plan
            .register_view(view)
            .map_err(WorthUiQueryViewRegistrationError::Binding)?;
        let descriptor = visible_state_bindings.into_iter().fold(
            crate::capability::ViewBindingDescriptor::from_definition(id, family, definition)
                .with_denial_presentation(denial_presentation),
            crate::capability::ViewBindingDescriptor::with_visible_state_binding,
        );
        self.inner = self.inner.register_view_binding(descriptor);
        Ok(self)
    }

    pub fn register_scalar_projection(
        mut self,
        registration: worth_ui_query_binding::UiScalarProjectionRegistration,
    ) -> Result<Self, WorthUiProjectionRegistrationError> {
        self.query_binding_plan = self
            .query_binding_plan
            .register_scalar_projection(registration)
            .map_err(WorthUiProjectionRegistrationError)?;
        Ok(self)
    }

    pub fn register_application_scalar_projection(
        mut self,
        registration: worth_ui_query_binding::UiApplicationScalarProjectionRegistration,
    ) -> Result<Self, WorthUiProjectionRegistrationError> {
        self.query_binding_plan = self
            .query_binding_plan
            .register_application_scalar_projection(registration)
            .map_err(WorthUiProjectionRegistrationError)?;
        Ok(self)
    }

    pub fn register_collection_projection(
        mut self,
        registration: worth_ui_query_binding::UiCollectionProjectionRegistration,
    ) -> Result<Self, WorthUiProjectionRegistrationError> {
        self.query_binding_plan = self
            .query_binding_plan
            .register_collection_projection(registration)
            .map_err(WorthUiProjectionRegistrationError)?;
        Ok(self)
    }
}
