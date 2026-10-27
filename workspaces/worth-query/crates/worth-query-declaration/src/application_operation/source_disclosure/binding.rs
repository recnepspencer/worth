use crate::application_query::{ApplicationQueryBinding, ApplicationQueryParameterSet};
use crate::application_schema::ApplicationSchema;

use super::super::ApplicationMutationBinding;

/// Original parameters cannot be captured under this source's declared meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationSourceDisclosureParameterDenial {
    InvalidOriginalInput,
    InvalidDisclosureParameters,
}

/// Maps one actual source input to its independently installed inspection query.
/// The mapping runs during original admission; later inspection re-admits the
/// captured parameters and never reconstructs them from receipt display fields.
pub trait ApplicationSourceDisclosureBinding<Schema: ApplicationSchema>: Sized + 'static {
    const IDENTITY: &'static str;
    type Source: ApplicationMutationBinding<Schema>;
    type Disclosure: ApplicationQueryBinding<Schema>;

    fn disclosure_parameters(
        original: &<Self::Source as ApplicationMutationBinding<Schema>>::Input,
    ) -> Result<
        ApplicationQueryParameterSet<<Self::Disclosure as ApplicationQueryBinding<Schema>>::Query>,
        ApplicationSourceDisclosureParameterDenial,
    >;
}
