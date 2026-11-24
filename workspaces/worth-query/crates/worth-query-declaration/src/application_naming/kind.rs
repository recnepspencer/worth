//! A kind marker's persistent-naming declaration: the subject its names
//! belong to and the entity that owns each named record.

use super::{ApplicationNameSubjectKind, NameOwnerScope};
use crate::application_schema::fragment;

/// Declares kind `Self` as a persistent-name kind.
///
/// `Subject` states whether `Self` is a declared entity or relation, so the
/// subject name is the marker's own declared `NAME` and can never drift from
/// it. `OwnerScope` states which entity owns each named record
/// ([`super::Unowned`] when none does). Stored names, owner-scoped resolution
/// and the owners reported with an ambiguous resolution all read this one
/// declaration; nothing is ever inferred from a name's text.
///
/// ```
/// use worth_query_declaration::facade::application_naming::{
///     ApplicationNameSubjectKind, EntityNameSubject, NameSubjectRole, Unowned,
///     WorthQueryNamedKind,
/// };
/// use worth_query_declaration::facade::application_schema::fragment::Entity;
///
/// struct Face;
/// impl Entity for Face {
///     const NAME: &'static str = "Face";
///     type Aspects = ();
///     type Relations = ();
/// }
/// impl WorthQueryNamedKind for Face {
///     type Subject = EntityNameSubject;
///     type OwnerScope = Unowned;
/// }
///
/// assert_eq!(<EntityNameSubject as NameSubjectRole<Face>>::SUBJECT_NAME, "Face");
/// assert_eq!(
///     <EntityNameSubject as NameSubjectRole<Face>>::SUBJECT_KIND,
///     ApplicationNameSubjectKind::Entity
/// );
/// ```
///
/// A marker that is not a declared relation cannot claim the relation
/// subject:
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_naming::{
///     RelationNameSubject, Unowned, WorthQueryNamedKind,
/// };
/// use worth_query_declaration::facade::application_schema::fragment::Entity;
///
/// struct Face;
/// impl Entity for Face {
///     const NAME: &'static str = "Face";
///     type Aspects = ();
///     type Relations = ();
/// }
/// impl WorthQueryNamedKind for Face {
///     type Subject = RelationNameSubject;
///     type OwnerScope = Unowned;
/// }
/// ```
pub trait WorthQueryNamedKind: Sized + 'static {
    /// [`EntityNameSubject`] or [`RelationNameSubject`].
    type Subject: NameSubjectRole<Self>;
    /// [`super::Unowned`] or [`super::OwnedThrough`] a declared relation chain.
    type OwnerScope: NameOwnerScope<Self>;
}

/// The subject class and declared subject name of kind `K`.
pub trait NameSubjectRole<K> {
    const SUBJECT_KIND: ApplicationNameSubjectKind;
    const SUBJECT_NAME: &'static str;
}

/// The subject of a declared entity kind: its `Entity::NAME`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntityNameSubject;

/// The subject of a declared relation kind: its `Relation::NAME`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationNameSubject;

impl<K: fragment::Entity> NameSubjectRole<K> for EntityNameSubject {
    const SUBJECT_KIND: ApplicationNameSubjectKind = ApplicationNameSubjectKind::Entity;
    const SUBJECT_NAME: &'static str = K::NAME;
}

impl<K: fragment::Relation> NameSubjectRole<K> for RelationNameSubject {
    const SUBJECT_KIND: ApplicationNameSubjectKind = ApplicationNameSubjectKind::Relation;
    const SUBJECT_NAME: &'static str = K::NAME;
}

/// The subject class of kind `K`'s names.
pub(crate) const fn subject_kind_of<K: WorthQueryNamedKind>() -> ApplicationNameSubjectKind {
    <K::Subject as NameSubjectRole<K>>::SUBJECT_KIND
}

/// The declared subject name of kind `K`'s names.
pub(crate) const fn subject_name_of<K: WorthQueryNamedKind>() -> &'static str {
    <K::Subject as NameSubjectRole<K>>::SUBJECT_NAME
}
