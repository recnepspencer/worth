use crate::application_query::ApplicationQueryBindingReference;
use crate::application_schema::ApplicationSchema;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::super::ApplicationMutationBinding;
use super::ApplicationSourceDisclosureBinding;

/// Portable source/query association. Installation must resolve the actual types.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredApplicationSourceDisclosure {
    parts: DeclaredApplicationSourceDisclosureParts,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredApplicationSourceDisclosureParts {
    pub identity: WorthQueryPortableTypeIdentity,
    pub source_binding: WorthQueryPortableTypeIdentity,
    pub query: ApplicationQueryBindingReference,
}

impl DeclaredApplicationSourceDisclosure {
    pub fn of<Schema: ApplicationSchema, Binding: ApplicationSourceDisclosureBinding<Schema>>(
    ) -> Self {
        Self::from_untrusted_parts(DeclaredApplicationSourceDisclosureParts {
            identity: WorthQueryPortableTypeIdentity::declared(Binding::IDENTITY),
            source_binding: WorthQueryPortableTypeIdentity::declared(
                <Binding::Source as ApplicationMutationBinding<Schema>>::IDENTITY,
            ),
            query: ApplicationQueryBindingReference::of::<Schema, Binding::Disclosure>(),
        })
    }

    pub fn from_untrusted_parts(parts: DeclaredApplicationSourceDisclosureParts) -> Self {
        Self { parts }
    }

    pub fn parts(&self) -> &DeclaredApplicationSourceDisclosureParts {
        &self.parts
    }
}
