use super::super::{DeclaredPreImageDemand, DeclaredPreImageLocus};
use crate::application_operation::{
    ApplicationMutationBinding, ApplicationMutationDescription, ApplicationMutationIntent,
};
use crate::application_schema::{
    ApplicationDeclaredOperation, ApplicationOperationDefinitionBuilder, ApplicationSchema,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

pub type ApplicationInverseBinding<Schema, Correction> =
    <<Correction as ApplicationCorrectionBinding<Schema>>::InverseIntent as ApplicationMutationIntent<Schema>>::Binding;

/// Read-only original values; this view grants no current read or write authority.
pub trait ApplicationCorrectionPreImage<Schema> {
    fn field(
        &self,
        locus: &DeclaredPreImageLocus<Schema>,
    ) -> Option<&worth_foundational::facade::AspectValue>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationCorrectionDerivationDenial {
    MissingPreImage,
    InvalidPreImage,
    InvalidOriginalInput,
    UnsupportedSourceMeaning,
    InvalidDisclosureParameters,
}

/// One installed source operation's deterministic recorded inverse association.
pub trait ApplicationCorrectionBinding<Schema: ApplicationSchema>: Sized + 'static {
    const IDENTITY: &'static str;
    type Source: ApplicationMutationBinding<Schema, Operation: ApplicationDeclaredOperation<Schema>>;
    type InverseIntent: ApplicationMutationIntent<Schema>;
    /// Current read authority for this binding's retained input and pre-images.
    type Disclosure: crate::application_query::ApplicationQueryBinding<Schema>;

    /// Captured while the original operation is admitted, before history becomes protected.
    fn disclosure_parameters(
        original: &<Self::Source as ApplicationMutationBinding<Schema>>::Input,
    ) -> Result<
        crate::application_query::ApplicationQueryParameterSet<
            <Self::Disclosure as crate::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        ApplicationCorrectionDerivationDenial,
    >;

    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        Schema,
        <Self::Source as ApplicationMutationBinding<Schema>>::Operation,
        <Self::Source as ApplicationMutationBinding<Schema>>::Input,
        true,
        false,
    >;
    fn preimage_demand() -> DeclaredPreImageDemand<Schema>;
    fn derive_inverse(
        original: &<Self::Source as ApplicationMutationBinding<Schema>>::Input,
        preimage: &impl ApplicationCorrectionPreImage<Schema>,
    ) -> Result<Self::InverseIntent, ApplicationCorrectionDerivationDenial>;
    fn inverse_key(key: &str) -> <ApplicationInverseBinding<Schema, Self> as ApplicationMutationBinding<Schema>>::IdempotencyKey;
    fn visit<Visitor: super::ApplicationCorrectionVisitor<Schema>>(
        dispatch: super::ApplicationCorrectionDispatch<Schema, Self, Visitor>,
    ) -> Visitor::Output;
}

/// Portable descriptive association derived from both actual mutation bindings.
/// Installation resolves this against the schema's native binding inventory.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredInverseBinding {
    identity: WorthQueryPortableTypeIdentity,
    source: ApplicationMutationDescription,
    inverse: ApplicationMutationDescription,
    source_handler: String,
    inverse_handler: String,
    source_query: Option<String>,
    inverse_query: Option<String>,
    disclosure: super::DeclaredCorrectionDisclosure,
}

impl DeclaredInverseBinding {
    /// Reconstructs descriptive meaning; native installation validates the association.
    pub fn from_untrusted_parts(parts: DeclaredInverseBindingParts) -> Self {
        Self {
            identity: parts.identity,
            source: parts.source,
            inverse: parts.inverse,
            source_handler: parts.source_handler,
            inverse_handler: parts.inverse_handler,
            source_query: parts.source_query,
            inverse_query: parts.inverse_query,
            disclosure: parts.disclosure,
        }
    }

    pub fn of<Schema: ApplicationSchema, Correction: ApplicationCorrectionBinding<Schema>>() -> Self
    {
        let source = Correction::Source::descriptor();
        let inverse = ApplicationInverseBinding::<Schema, Correction>::descriptor();
        Self {
            identity: WorthQueryPortableTypeIdentity::declared(Correction::IDENTITY),
            source: source.description().clone(),
            inverse: inverse.description().clone(),
            source_handler: source.handler().identity().to_owned(),
            inverse_handler: inverse.handler().identity().to_owned(),
            source_query: source.source_query_identifier().map(str::to_owned),
            inverse_query: inverse.source_query_identifier().map(str::to_owned),
            disclosure: super::DeclaredCorrectionDisclosure::of::<Schema, Correction::Disclosure>(),
        }
    }

    pub fn identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.identity
    }
    pub fn source(&self) -> &ApplicationMutationDescription {
        &self.source
    }
    pub fn inverse(&self) -> &ApplicationMutationDescription {
        &self.inverse
    }
    pub fn source_handler(&self) -> &str {
        &self.source_handler
    }
    pub fn inverse_handler(&self) -> &str {
        &self.inverse_handler
    }
    pub fn source_query(&self) -> Option<&str> {
        self.source_query.as_deref()
    }
    pub fn inverse_query(&self) -> Option<&str> {
        self.inverse_query.as_deref()
    }
    pub fn disclosure(&self) -> &super::DeclaredCorrectionDisclosure {
        &self.disclosure
    }
}

/// Authority-free fields decoded from a portable inverse association.
pub struct DeclaredInverseBindingParts {
    pub identity: WorthQueryPortableTypeIdentity,
    pub source: ApplicationMutationDescription,
    pub inverse: ApplicationMutationDescription,
    pub source_handler: String,
    pub inverse_handler: String,
    pub source_query: Option<String>,
    pub inverse_query: Option<String>,
    pub disclosure: super::DeclaredCorrectionDisclosure,
}
