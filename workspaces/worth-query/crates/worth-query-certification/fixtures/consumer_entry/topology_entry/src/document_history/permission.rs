//! Real history grants share storage while typed actions seal each authority.
use super::super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_capability::*, application_schema::*, portable_identity::WorthQueryPortableType,
};
mod contract;
mod grant_status;
mod grants;
pub(super) use contract::contract;
pub use contract::{bind_navigation_grant, bind_retention_grants};
pub use grant_status::*;
pub use grants::*;
pub(super) trait HistoryPermission<S: TopologySchemaBinding>:
    ApplicationCapabilityMarkerIdentity<Schema = S>
{
    const ACTION: &'static str;
    const GRANT_KIND: &'static str;
}
macro_rules! context {
    ($name:ident, $identity:ident, $reference:ident) => {
        pub struct $name<S>(PhantomData<fn() -> S>);
        impl<S> WorthQueryPortableType for $name<S> {
            const PORTABLE_TYPE_NAME: &'static str = stringify!($name);
        }
        impl<S: TopologySchemaBinding> $identity for $name<S> {
            type Schema = S;
            const IDENTIFIER: &'static str = stringify!($name);
        }
        impl<S: TopologySchemaBinding> $name<S> {
            pub fn reference() -> $reference<S, Self> {
                $reference::from_declaration()
            }
        }
    };
}
context!(
    PlanarHistoryContext,
    ApplicationCapabilityContextMarkerIdentity,
    ApplicationCapabilityContextRef
);
context!(
    PlanarHistoryProvenance,
    ApplicationCapabilityProvenanceMarkerIdentity,
    ApplicationCapabilityProvenanceRef
);
impl<S: TopologySchemaBinding, C: HistoryPermission<S>> ApplicationCapabilityRequest<S, C>
    for PlanarOutputRead
{
    type Scope = Body;
    type Context = PlanarHistoryContext<S>;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<S, Body, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                BodyKey::reference(),
                encoded(self.body_key.clone()),
            ),
            encoded(C::ACTION.to_owned()),
            encoded("model-history".to_owned()),
            ApplicationCapabilityRequestContext::new(PlanarHistoryContext::<S>::reference()),
        ))
    }
}
pub(super) fn encoded(
    value: String,
) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value).expect("history permission values are strings")
}
pub(super) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    grant_status::declare(grants::declare(schema))
        .capability_context(PlanarHistoryContext::<S>::reference())
        .capability_provenance(PlanarHistoryProvenance::<S>::reference())
}
