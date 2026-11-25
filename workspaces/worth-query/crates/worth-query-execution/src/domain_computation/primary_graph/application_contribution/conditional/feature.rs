//! Feature root that admits one conditional binding's operation into a program.
use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_program::{
        ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureSpec,
    },
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryApplicationConditionalBinding;

/// Program feature root owned by conditional binding `Binding`.
///
/// Its identity is the binding's identity and its only action is the binding's
/// conditional operation, so a program declares a conditional binding once
/// instead of hand-writing a matching feature and feature specification.
pub struct WorthQueryConditionalFeature<Binding>(PhantomData<fn() -> Binding>);

impl<Schema, Binding> ApplicationFeature<Schema> for WorthQueryConditionalFeature<Binding>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationConditionalBinding<Schema>,
{
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = Binding::IDENTITY;
}

impl<Binding> WorthQueryConditionalFeature<Binding> {
    /// The root feature specification admitting `Binding::Operation`.
    pub fn feature_spec<Schema>() -> ApplicationFeatureSpec
    where
        Schema: ApplicationSchema + 'static,
        Binding: WorthQueryApplicationConditionalBinding<Schema>,
        Binding::Operation: ApplicationOperationMarkerIdentity<Schema>,
    {
        ApplicationFeatureSpec::root::<Schema, Self>()
            .conditional_operation::<Binding::Operation>()
            .finish()
    }
}
