use std::marker::PhantomData;

use super::ApplicationResourceBinding;
use crate::application_schema::ApplicationSchema;

/// Typed selection of installed meaning, never a live request or cleanup owner.
pub struct ApplicationResourceRef<Schema, Binding> {
    _binding: PhantomData<fn() -> (Schema, Binding)>,
}

impl<Schema: ApplicationSchema, Binding: ApplicationResourceBinding<Schema>>
    ApplicationResourceRef<Schema, Binding>
{
    pub const fn from_declaration() -> Self {
        Self {
            _binding: PhantomData,
        }
    }
}
