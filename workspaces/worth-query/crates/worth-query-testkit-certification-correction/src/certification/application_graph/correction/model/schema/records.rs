//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct NumericModel;
impl Fragment for NumericModel {
    const NAME: &'static str = "numeric_model";
    type Bindings = NumericModelBindings;
    type Members = (ExternalMapping, Principal, Number, Grant);
}
pub struct NumericModelBindings;
impl BindingSet for NumericModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<BinaryCounter> for NumericModelBindings {
    type Binding = BrokenBinaryBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
impl Scalar<CanonicalCounter> for NumericModelBindings {
    type Binding = CanonicalBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
impl Scalar<DelegatedCounter> for NumericModelBindings {
    type Binding = DelegatedBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for NumericModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for NumericModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for NumericModelBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<String> for NumericModelBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<Counter> for NumericModelBindings {
    type Binding = InlineCounterBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
impl Scalar<bool> for NumericModelBindings {
    type Binding = BoolApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}

#[path = "records/entity_members.rs"]
mod entity_members;
pub use entity_members::{ExternalMapping, Grant, Number, Principal};
#[path = "records/value_rows.rs"]
mod value_rows;
pub use value_rows::{
    DefinitionRevisionFacts, DefinitionRevisionFactsValue, ExternalIdentity, ExternalIdentityValue,
    GrantFacts, GrantFactsValue, InputFacts, InputFactsValue, NumberFacts, NumberFactsValue,
    PrincipalFacts, PrincipalFactsValue, ScalarFacts, ScalarFactsValue, ScalarResult,
    ScalarResultValue, WriteFacts, WriteFactsValue,
};
#[path = "records/selections.rs"]
mod selections;
pub use selections::{
    Bias, Binary, Calculated, Canonical, Ceiling, DefinitionRevision, Delegated, Disclosure,
    ExternalIdentityField, GrantAction, GrantDepth, GrantIdentity, GrantNotAfter, GrantNotBefore,
    GrantPurpose, GrantScopeKey, GrantStatus, Key, MappingStatusField, Operand, Primary,
    PrincipalIdentityField, ReadEnabled, Secondary, Sibling, Total, Value,
};
#[path = "records/relations.rs"]
mod relations;
pub use relations::{
    AuthoredReference, GrantGrantee, GrantGrantor, GrantParent, GrantResource, MappingTarget,
};
