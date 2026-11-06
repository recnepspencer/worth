//! One record model owns field lowering and reusable scalar bindings.
#[path = "annotation_text.rs"]
mod annotation_text;
pub use annotation_text::{AnnotationText, AnnotationTextBinding};
#[path = "elevation_key.rs"]
mod elevation_key;
pub use elevation_key::{ElevationKey, ElevationKeyBinding};
#[path = "grant_key.rs"]
mod grant_key;
use super::super::capability::*;
use super::*;
pub use grant_key::{GrantKey, GrantKeyBinding};
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct ExecutionRecords;
impl fragment::Fragment for ExecutionRecords {
    const NAME: &'static str = "execution_records";
    type Bindings = ExecutionRecordsBindings;
    type Members = (
        ExternalMapping,
        Account,
        Activity,
        CapabilityReview,
        CapabilityActionRecord,
    );
}
pub struct ExecutionRecordsBindings;
impl fragment::BindingSet for ExecutionRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for ExecutionRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<u64> for ExecutionRecordsBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for ExecutionRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<String> for ExecutionRecordsBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<CapabilityElevationStatus> for ElevationBindings {
    type Binding = CapabilityElevationStatusBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<CapabilityReviewKind> for ExecutionRecordsBindings {
    type Binding = CapabilityReviewKindBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<CapabilityReviewStatus> for ExecutionRecordsBindings {
    type Binding = CapabilityReviewStatusBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<CapabilityAction> for GrantBindings {
    type Binding = CapabilityActionBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<CapabilityPurpose> for GrantBindings {
    type Binding = CapabilityPurposeBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<CapabilityDisclosure> for GrantBindings {
    type Binding = CapabilityDisclosureBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<CapabilityStatus> for GrantBindings {
    type Binding = CapabilityStatusBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}

#[path = "records/entities.rs"]
mod entities;
pub use entities::*;

#[path = "records/value_rows.rs"]
mod value_rows;
pub use value_rows::*;

#[path = "records/capability_rows.rs"]
mod capability_rows;
pub use capability_rows::*;

#[path = "records/selections.rs"]
mod selections;
pub use selections::*;

#[path = "records/relations.rs"]
mod relations;
pub use relations::*;

#[path = "records/role_bindings.rs"]
mod role_bindings;
pub use role_bindings::*;
