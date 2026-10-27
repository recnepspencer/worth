use crate::application_query::{ApplicationQueryBinding, ApplicationQueryParameterSet};
use crate::application_schema::{ApplicationReadableScalarValueBinding, ApplicationSchema};
use crate::portable_identity::WorthQueryPortableType;
mod descriptor;
pub use descriptor::ApplicationResourceBindingDescriptor;

pub type ApplicationResourceInput<Schema, Binding> = <<Binding as ApplicationResourceBinding<
    Schema,
>>::InputBinding as crate::application_schema::ApplicationScalarValueBinding>::Value;

/// One resource's actual typed recipes. Access supplies principal/scope meaning;
/// its ordinary query permission never grants a resource action.
pub trait ApplicationResourceBinding<Schema>: WorthQueryPortableType + Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Access: ApplicationQueryBinding<Schema>;
    type InputBinding: ApplicationReadableScalarValueBinding;
    type ResultBinding: ApplicationReadableScalarValueBinding;
    type FailureBinding: ApplicationReadableScalarValueBinding;
    type Provider: WorthQueryPortableType;

    const IDENTITY: &'static str;

    fn reference() -> super::ApplicationResourceRef<Schema, Self> {
        super::ApplicationResourceRef::from_declaration()
    }

    /// Called once by start, under its original request, before retaining input.
    fn scope(
        input: &ApplicationResourceInput<Schema, Self>,
    ) -> <Self::Access as ApplicationQueryBinding<Schema>>::ScopeBinding;

    /// Called once; subsequent actions reuse the admitted original parameters.
    fn parameters(
        input: &ApplicationResourceInput<Schema, Self>,
    ) -> ApplicationQueryParameterSet<<Self::Access as ApplicationQueryBinding<Schema>>::Query>;
}
