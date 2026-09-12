use std::any::TypeId;

use super::{ApplicationFieldBindingLocus, ApplicationFieldBindingRecipe};
use crate::application_operation::ApplicationMutationBindingDescriptor;
use crate::application_query::ApplicationQueryBindingDescriptor;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
struct DeclaredApplicationMemberMarker {
    name: String,
    value_identity: WorthQueryPortableTypeIdentity,
    marker_type: TypeId,
    value_type: TypeId,
}

impl DeclaredApplicationMemberMarker {
    fn of<Marker: 'static, Value: 'static>(
        name: &'static str,
        value_identity: WorthQueryPortableTypeIdentity,
    ) -> Self {
        Self {
            name: name.to_owned(),
            value_identity,
            marker_type: TypeId::of::<Marker>(),
            value_type: TypeId::of::<Value>(),
        }
    }

    fn matches<Marker: 'static, Value: 'static>(
        &self,
        name: &str,
        value_identity: &WorthQueryPortableTypeIdentity,
    ) -> bool {
        self.name == name
            && &self.value_identity == value_identity
            && self.marker_type == TypeId::of::<Marker>()
            && self.value_type == TypeId::of::<Value>()
    }
}

/// Compiler-local declaration provenance for typed operation and effect members.
///
/// This sidecar never enters canonical or portable package meaning. It binds
/// typed authoring and installed lookup to the marker types selected by the
/// owning [`super::ApplicationSchema::declaration`] implementation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApplicationSchemaMemberProvenance {
    field_bindings: Vec<ApplicationFieldBindingRecipe>,
    conflicting_field_binding: bool,
    operations: Vec<DeclaredApplicationMemberMarker>,
    effects: Vec<DeclaredApplicationMemberMarker>,
    mutation_bindings: Vec<ApplicationMutationBindingDescriptor>,
    query_bindings: Vec<ApplicationQueryBindingDescriptor>,
}

impl ApplicationSchemaMemberProvenance {
    #[doc(hidden)]
    pub fn is_empty(&self) -> bool {
        self.field_bindings.is_empty()
            && self.operations.is_empty()
            && self.effects.is_empty()
            && self.mutation_bindings.is_empty()
            && self.query_bindings.is_empty()
    }

    pub(super) fn register_field_binding(&mut self, recipe: ApplicationFieldBindingRecipe) {
        if let Some(existing) = self
            .field_bindings
            .iter()
            .find(|existing| existing.locus() == recipe.locus())
        {
            self.conflicting_field_binding |= !existing.has_same_contract(&recipe);
            return;
        }
        self.field_bindings.push(recipe);
    }

    pub(super) fn register_operation<Operation: 'static, Input: 'static>(
        &mut self,
        name: &'static str,
        input_identity: WorthQueryPortableTypeIdentity,
    ) {
        self.operations
            .push(DeclaredApplicationMemberMarker::of::<Operation, Input>(
                name,
                input_identity,
            ));
    }

    pub(super) fn register_effect<Effect: 'static, Payload: 'static>(
        &mut self,
        name: &'static str,
        payload_identity: WorthQueryPortableTypeIdentity,
    ) {
        self.effects
            .push(DeclaredApplicationMemberMarker::of::<Effect, Payload>(
                name,
                payload_identity,
            ));
    }

    pub(super) fn register_query_binding(&mut self, descriptor: ApplicationQueryBindingDescriptor) {
        self.query_bindings.push(descriptor);
    }

    pub(super) fn register_mutation_binding(
        &mut self,
        descriptor: ApplicationMutationBindingDescriptor,
    ) {
        self.mutation_bindings.push(descriptor);
    }

    pub(super) fn normalize(&mut self) {
        let order = |left: &DeclaredApplicationMemberMarker,
                     right: &DeclaredApplicationMemberMarker| {
            (left.name.as_str(), left.value_identity.as_str())
                .cmp(&(right.name.as_str(), right.value_identity.as_str()))
        };
        self.operations.sort_by(order);
        self.effects.sort_by(order);
        self.field_bindings
            .sort_by(|left, right| left.locus().cmp(right.locus()));
        self.query_bindings
            .sort_by(|left, right| left.identity().cmp(right.identity()));
        self.mutation_bindings
            .sort_by(|left, right| left.identity().cmp(right.identity()));
    }

    pub(super) const fn has_conflicting_field_binding(&self) -> bool {
        self.conflicting_field_binding
    }

    pub(super) fn field_bindings_match(&self, members: &[super::ApplicationSchemaMember]) -> bool {
        self.field_bindings
            .iter()
            .all(|recipe| members.iter().any(|member| recipe.matches_member(member)))
    }

    pub fn field_bindings(&self) -> &[ApplicationFieldBindingRecipe] {
        &self.field_bindings
    }

    pub fn field_binding(
        &self,
        locus: &ApplicationFieldBindingLocus,
    ) -> Option<&ApplicationFieldBindingRecipe> {
        self.field_bindings
            .iter()
            .find(|recipe| recipe.locus() == locus)
    }

    pub fn query_bindings(&self) -> &[ApplicationQueryBindingDescriptor] {
        &self.query_bindings
    }

    pub fn mutation_bindings(&self) -> &[ApplicationMutationBindingDescriptor] {
        &self.mutation_bindings
    }

    #[doc(hidden)]
    pub fn admits_mutation_binding_operation(
        &self,
        descriptor: &ApplicationMutationBindingDescriptor,
    ) -> bool {
        self.operations.iter().any(|member| {
            member.name == descriptor.operation_name()
                && member.value_identity == *descriptor.input_identity()
                && member.marker_type == descriptor.operation_type()
                && member.value_type == descriptor.input_type()
        })
    }

    #[doc(hidden)]
    pub fn admits_operation<Operation: 'static, Input: 'static>(
        &self,
        name: &str,
        input_identity: &WorthQueryPortableTypeIdentity,
    ) -> bool {
        self.operations
            .iter()
            .any(|member| member.matches::<Operation, Input>(name, input_identity))
    }

    #[doc(hidden)]
    pub fn admits_effect<Effect: 'static, Payload: 'static>(
        &self,
        name: &str,
        payload_identity: &WorthQueryPortableTypeIdentity,
    ) -> bool {
        self.effects
            .iter()
            .any(|member| member.matches::<Effect, Payload>(name, payload_identity))
    }
}
