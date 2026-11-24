//! Descriptive contacts of a managed predecessor read.
use crate::identity::data::KindId;
use worth_foundational::facade::{AspectKey, FieldKey};

/// What a managed Parent reads beside its Source program: its projected,
/// filtered and ordered fields, its nested result relations and their
/// endpoint kinds, and the scope-key field that resolves the probed scope.
/// A field write is relevant on any entity of its kind, so renaming another
/// scope to the probed key is never hidden.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ManagedIndexParentReadFootprint {
    pub fields: Vec<(KindId, AspectKey, FieldKey)>,
    pub relations: Vec<KindId>,
    pub entities: Vec<KindId>,
    /// The scope-key field: a scope appearing with the probed scope's value
    /// would make that key resolve to another scope.
    pub scope_key: Option<(KindId, AspectKey, FieldKey)>,
}
