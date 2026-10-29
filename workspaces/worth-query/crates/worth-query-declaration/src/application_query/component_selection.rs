//! Complete component result meaning. Native alone owns identity and membership.
use super::{ApplicationQueryMarkerIdentity, ApplicationQueryResultSlotKey};
use crate::{
    application_schema::ApplicationComponentSetRef,
    portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity},
};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryComponentSelection {
    OfScopeMember,
    Census,
}
impl ApplicationQueryComponentSelection {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::OfScopeMember => "of-scope-member",
            Self::Census => "census",
        }
    }
}

/// One disclosure slot covers Native identity, complete membership and count.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryComponentMeaning {
    query: WorthQueryPortableTypeIdentity,
    slot: WorthQueryPortableTypeIdentity,
    set: String,
    member: String,
    output_name: String,
    selection: ApplicationQueryComponentSelection,
}
/// Descriptive input; fresh declaration and complete-schema validation are required.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationQueryComponentParts {
    pub query: WorthQueryPortableTypeIdentity,
    pub slot: WorthQueryPortableTypeIdentity,
    pub set: String,
    pub member: String,
    pub output_name: String,
    pub selection: ApplicationQueryComponentSelection,
}
impl ApplicationQueryComponentMeaning {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationQueryComponentParts) -> Self {
        Self {
            query: parts.query,
            slot: parts.slot,
            set: parts.set,
            member: parts.member,
            output_name: parts.output_name,
            selection: parts.selection,
        }
    }
    pub const fn query_type(&self) -> &str {
        self.query.as_str()
    }
    pub const fn slot_type(&self) -> &str {
        self.slot.as_str()
    }
    pub fn query_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.query.clone()
    }
    pub fn slot_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.slot.clone()
    }
    pub fn set(&self) -> &str {
        &self.set
    }
    pub fn member(&self) -> &str {
        &self.member
    }
    pub fn output_name(&self) -> &str {
        &self.output_name
    }
    pub const fn selection(&self) -> ApplicationQueryComponentSelection {
        self.selection
    }
    pub fn slot_key(&self) -> ApplicationQueryResultSlotKey {
        ApplicationQueryResultSlotKey::components(self)
    }
    /// Shared pure identity vocabulary; each canonical artifact owns its prefix.
    pub fn visit_canonical_parts(&self, mut visit: impl FnMut(&str, &str)) {
        drive_component_parts(self, &mut visit);
    }
}

#[inline(never)]
fn drive_component_parts(
    meaning: &ApplicationQueryComponentMeaning,
    visit: &mut dyn FnMut(&str, &str),
) {
    for (key, value) in [
        ("query", meaning.query_type()),
        ("slot", meaning.slot_type()),
        ("set", meaning.set()),
        ("member", meaning.member()),
        ("output", meaning.output_name()),
        ("selection", meaning.selection.canonical_name()),
    ] {
        visit(key, value);
    }
}

pub struct ApplicationQueryComponentRef<Schema, Query, Slot, Set, Entity> {
    meaning: ApplicationQueryComponentMeaning,
    _marker: PhantomData<fn() -> (Schema, Query, Slot, Set, Entity)>,
}
impl<Schema, Query, Slot, Set, Entity>
    ApplicationQueryComponentRef<Schema, Query, Slot, Set, Entity>
where
    Query: ApplicationQueryMarkerIdentity<Schema>,
    Slot: WorthQueryPortableType,
{
    pub fn census(
        output_name: &'static str,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
    ) -> Self {
        Self::declared(output_name, set, ApplicationQueryComponentSelection::Census)
    }
    fn declared(
        output_name: &'static str,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
        selection: ApplicationQueryComponentSelection,
    ) -> Self {
        Self {
            meaning: ApplicationQueryComponentMeaning::from_untrusted_parts(
                WorthQueryPortableApplicationQueryComponentParts {
                    query: WorthQueryPortableTypeIdentity::declared(Query::QUERY_TYPE_NAME),
                    slot: Slot::PORTABLE_TYPE_IDENTITY,
                    set: set.name().into(),
                    member: set.member().into(),
                    output_name: output_name.into(),
                    selection,
                },
            ),
            _marker: PhantomData,
        }
    }
}
impl<Schema, Query, Slot, Set, Entity>
    ApplicationQueryComponentRef<Schema, Query, Slot, Set, Entity>
where
    Query: ApplicationQueryMarkerIdentity<Schema, Scope = Entity>,
    Slot: WorthQueryPortableType,
{
    pub fn of_scope_member(
        output_name: &'static str,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
    ) -> Self {
        Self::declared(
            output_name,
            set,
            ApplicationQueryComponentSelection::OfScopeMember,
        )
    }
}
impl<Schema, Query, Slot, Set, Entity>
    ApplicationQueryComponentRef<Schema, Query, Slot, Set, Entity>
{
    pub fn meaning(&self) -> &ApplicationQueryComponentMeaning {
        &self.meaning
    }
    pub fn slot_key(&self) -> ApplicationQueryResultSlotKey {
        self.meaning.slot_key()
    }
    pub(crate) fn into_meaning(self) -> ApplicationQueryComponentMeaning {
        self.meaning
    }
}
