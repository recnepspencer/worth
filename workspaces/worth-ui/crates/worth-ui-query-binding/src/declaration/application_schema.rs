#[path = "application_schema/records.rs"]
pub(in crate::declaration) mod storage_model;
pub use storage_model::{
    CollectionItemAspect, CollectionItemKeyField, CollectionItemStatusField, IdentityAspect,
    IdentityIdField, MeasurementAspect, MeasurementValueField, QueryRevisionAspect,
    QueryRevisionValueField, QueryTextAspect, QueryTextStatusField, SizeAspect, SizeValueField,
    UiRecords, WorthUiRecord,
};
use worth_foundational::facade::CanonicalF32;

use worth_query_decl::facade::{worth_query_portable_type, worth_query_value_binding};

use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct WorthUiApplicationSchema;
impl Fragment for WorthUiApplicationSchema {
    const NAME: &'static str = "WorthUiApplicationSchema";
    type Bindings = NoBindings;
    type Members = (WorthUiRecordContribution,);
}
impl ApplicationSchema for WorthUiApplicationSchema {
    const OWNER: &'static str = "worth_ui";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 1;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}

impl ApplicationSchemaComposition for WorthUiApplicationSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct WorthUiRecordContribution;
impl ApplicationSchemaContribution<WorthUiApplicationSchema> for WorthUiRecordContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.ui.record.v1");
    type Members = (UiRecords, Contracts);
}
impl WorthUiRecordContribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<WorthUiApplicationSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<WorthUiApplicationSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<WorthUiApplicationSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<WorthUiApplicationSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok({
            let schema = schema
                .principal_binding(super::WorthUiPrincipalBinding::reference())
                .invariant(super::status_integrity_invariant())
                .effect(super::WorthUiStatusChangedEffect::reference())
                .application_query(super::status_query_definition())
                .application_query_binding::<super::WorthUiStatusQueryBinding>();
            super::declare_status_action(super::declare_status_mutation(schema))
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct UiMeasurementValue(CanonicalF32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct UiSizeValue(CanonicalF32);

worth_query_portable_type!(
    UiMeasurementValue => "worth.ui.query-binding.measurement-value.v1"
);
worth_query_portable_type!(UiSizeValue => "worth.ui.query-binding.size-value.v1");

macro_rules! float_application_value_api {
    ($type:ty) => {
        impl $type {
            pub fn from_f32(value: f32) -> Self {
                Self(CanonicalF32::from_f32(value))
            }

            pub fn as_f32(self) -> f32 {
                self.0.as_f32()
            }
        }
    };
}

float_application_value_api!(UiMeasurementValue);
float_application_value_api!(UiSizeValue);

macro_rules! float_application_binding {
    ($binding:ident, $value:ident, $identity:literal) => {
        impl $value {
            fn encode_scalar(&self) -> CanonicalF32 {
                self.0
            }

            fn decode_scalar(value: CanonicalF32) -> Option<Self> {
                Some(Self(value))
            }
        }

        worth_query_value_binding! {
            pub $binding for $value {
                identity: $identity,
                scalar: Float32,
                encode: $value::encode_scalar,
                decode: $value::decode_scalar,
            }
        }
    };
}

float_application_binding!(
    UiMeasurementValueBinding,
    UiMeasurementValue,
    "worth.ui.query-binding.measurement-value.v1"
);
float_application_binding!(
    UiSizeValueBinding,
    UiSizeValue,
    "worth.ui.query-binding.size-value.v1"
);
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum WorthUiProjectionField {
    IdentityId,
    QueryTextStatus,
    QueryRevisionValue,
    CollectionItemStatus,
    CollectionItemKey,
    MeasurementValue,
    SizeValue,
}

impl WorthUiProjectionField {
    pub(crate) const fn native_key(self) -> &'static str {
        match self {
            Self::IdentityId => "id",
            Self::QueryTextStatus => "status",
            Self::CollectionItemStatus => "status",
            Self::CollectionItemKey => "key",
            Self::QueryRevisionValue | Self::MeasurementValue | Self::SizeValue => "value",
        }
    }

    pub(crate) const fn collection_contract_key(self) -> &'static str {
        match self {
            Self::IdentityId => "identity.id",
            Self::QueryTextStatus => "query_text.status",
            Self::QueryRevisionValue => "query_revision.value",
            Self::CollectionItemStatus => "collection_item.status",
            Self::CollectionItemKey => "collection_item.key",
            Self::MeasurementValue => "measurement.value",
            Self::SizeValue => "size.value",
        }
    }
}

pub trait WorthUiNativeField: sealed::Sealed {
    const FIELD: WorthUiProjectionField;
}

impl WorthUiNativeField for IdentityIdField {
    const FIELD: WorthUiProjectionField = WorthUiProjectionField::IdentityId;
}

impl WorthUiNativeField for QueryTextStatusField {
    const FIELD: WorthUiProjectionField = WorthUiProjectionField::QueryTextStatus;
}

impl WorthUiNativeField for QueryRevisionValueField {
    const FIELD: WorthUiProjectionField = WorthUiProjectionField::QueryRevisionValue;
}

impl WorthUiNativeField for CollectionItemStatusField {
    const FIELD: WorthUiProjectionField = WorthUiProjectionField::CollectionItemStatus;
}

impl WorthUiNativeField for CollectionItemKeyField {
    const FIELD: WorthUiProjectionField = WorthUiProjectionField::CollectionItemKey;
}

impl WorthUiNativeField for MeasurementValueField {
    const FIELD: WorthUiProjectionField = WorthUiProjectionField::MeasurementValue;
}

impl WorthUiNativeField for SizeValueField {
    const FIELD: WorthUiProjectionField = WorthUiProjectionField::SizeValue;
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::IdentityIdField {}
    impl Sealed for super::QueryTextStatusField {}
    impl Sealed for super::QueryRevisionValueField {}
    impl Sealed for super::CollectionItemStatusField {}
    impl Sealed for super::CollectionItemKeyField {}
    impl Sealed for super::MeasurementValueField {}
    impl Sealed for super::SizeValueField {}
}
