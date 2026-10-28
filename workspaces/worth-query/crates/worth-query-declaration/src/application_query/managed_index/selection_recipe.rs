//! Actual typed selector/codec correspondence retained by schema authoring.
use super::{
    ApplicationManagedIndexBinding, ApplicationManagedIndexBindingDescriptor,
    ApplicationManagedIndexProbeMeaning, ApplicationQueryManagedIndexMeaning,
};
use crate::application_query::ApplicationQueryBindingDescriptor;
use crate::application_schema::ApplicationSchema;
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use std::any::TypeId;

/// Compiler-local evidence of the parent query's selected probe codec.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationManagedIndexSelectionRecipe {
    query_name: String,
    query_identity: WorthQueryPortableTypeIdentity,
    query_type: TypeId,
    index_type: TypeId,
    probe_binding_type: TypeId,
    probe: ApplicationManagedIndexProbeMeaning,
    selection: ApplicationQueryManagedIndexMeaning,
}

impl ApplicationManagedIndexSelectionRecipe {
    pub(in crate::application_query) fn typed<
        S: ApplicationSchema,
        I: ApplicationManagedIndexBinding<S>,
        Q: 'static,
    >(
        name: &str,
        identity: WorthQueryPortableTypeIdentity,
        selection: ApplicationQueryManagedIndexMeaning,
    ) -> Self {
        Self {
            query_name: name.to_owned(),
            query_identity: identity,
            query_type: TypeId::of::<Q>(),
            index_type: TypeId::of::<I>(),
            probe_binding_type: TypeId::of::<I::ProbeBinding>(),
            probe: ApplicationManagedIndexProbeMeaning::of::<I::ProbeBinding>(),
            selection,
        }
    }

    pub fn query_name(&self) -> &str {
        &self.query_name
    }

    /// The full original typed query and selector must match the installed receiver.
    pub fn matches_parent(
        &self,
        parent: &ApplicationQueryBindingDescriptor,
        selection: &ApplicationQueryManagedIndexMeaning,
    ) -> bool {
        self.query_name == parent.query_name()
            && self.query_identity == *parent.query_identity()
            && self.query_type == parent.query_type()
            && &self.selection == selection
    }
}

impl ApplicationManagedIndexBindingDescriptor {
    /// Compare actual marker types AND the complete canonical probe profile.
    pub fn admits_selection(&self, selection: &ApplicationManagedIndexSelectionRecipe) -> bool {
        self.index_type() == selection.index_type
            && self.probe_binding_type() == selection.probe_binding_type
            && self.meaning().parts().probe == selection.probe
            && self.meaning().parts().name == selection.selection.parts().index
    }
}
