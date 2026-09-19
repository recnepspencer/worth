use worth_foundational::facade::CanonicalF32;
use worth_query_decl::facade::application_schema::{
    StringApplicationValueBinding, U64ApplicationValueBinding,
};
use worth_query_decl::facade::{
    worth_query_application, worth_query_application_contribution, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_portable_type, worth_query_value_binding,
};

worth_query_application! {
    pub WorthUiApplicationSchema {
        owner: "worth_ui",
        version: (1, 1),
        contributions: [WorthUiRecordContribution],
    }
}

worth_query_application_contribution! {
    pub contribution WorthUiRecordContribution in WorthUiApplicationSchema {
        identity: "worth.ui.record.v1",
        members: |schema| {
            let schema = schema
                .entity(WorthUiRecord::reference())
                .entity(super::WorthUiExternalPrincipalMapping::reference())
                .entity(super::WorthUiPrincipal::reference())
                .aspect(super::WorthUiExternalPrincipalMapping::reference(), super::WorthUiExternalIdentity::reference())
                .aspect(super::WorthUiPrincipal::reference(), super::WorthUiPrincipalIdentity::reference())
                .field(super::WorthUiExternalPrincipalMapping::reference(), super::WorthUiExternalIdentityKey::reference())
                .field(super::WorthUiExternalPrincipalMapping::reference(), super::WorthUiMappingStatus::reference())
                .field(super::WorthUiPrincipal::reference(), super::WorthUiPrincipalId::reference())
                .relation(super::WorthUiExternalPrincipal::reference(), super::WorthUiExternalPrincipalMapping::reference(), super::WorthUiPrincipal::reference())
                .principal_binding(super::WorthUiPrincipalBinding::reference())
                .aspect(WorthUiRecord::reference(), IdentityAspect::reference())
                .aspect(WorthUiRecord::reference(), QueryTextAspect::reference())
                .aspect(WorthUiRecord::reference(), QueryRevisionAspect::reference())
                .aspect(WorthUiRecord::reference(), CollectionItemAspect::reference())
                .aspect(WorthUiRecord::reference(), MeasurementAspect::reference())
                .aspect(WorthUiRecord::reference(), SizeAspect::reference())
                .field(WorthUiRecord::reference(), IdentityIdField::reference())
                .field(WorthUiRecord::reference(), QueryTextStatusField::reference())
                .field(WorthUiRecord::reference(), QueryRevisionValueField::reference())
                .field(WorthUiRecord::reference(), CollectionItemStatusField::reference())
                .field(WorthUiRecord::reference(), CollectionItemKeyField::reference())
                .field(WorthUiRecord::reference(), MeasurementValueField::reference())
                .field(WorthUiRecord::reference(), SizeValueField::reference())
                .invariant(super::status_integrity_invariant())
                .effect(super::WorthUiStatusChangedEffect::reference())
                .application_query(super::status_query_definition())
                .application_query_binding::<super::WorthUiStatusQueryBinding>();
            super::declare_status_action(super::declare_status_mutation(schema))
        }
    }
}

worth_query_entity!(pub WorthUiRecord for WorthUiApplicationSchema);
worth_query_aspect!(pub IdentityAspect for WorthUiApplicationSchema, WorthUiRecord; identity = AspectIdentity(0x91611056), revision = AspectContractRevision(1),);
worth_query_aspect!(pub QueryTextAspect for WorthUiApplicationSchema, WorthUiRecord; identity = AspectIdentity(0x91611057), revision = AspectContractRevision(1),);
worth_query_aspect!(pub QueryRevisionAspect for WorthUiApplicationSchema, WorthUiRecord; identity = AspectIdentity(0x91611058), revision = AspectContractRevision(1),);
worth_query_aspect!(pub CollectionItemAspect for WorthUiApplicationSchema, WorthUiRecord; identity = AspectIdentity(0x9161105b), revision = AspectContractRevision(1),);
worth_query_aspect!(pub MeasurementAspect for WorthUiApplicationSchema, WorthUiRecord; identity = AspectIdentity(0x91611059), revision = AspectContractRevision(1),);
worth_query_aspect!(pub SizeAspect for WorthUiApplicationSchema, WorthUiRecord; identity = AspectIdentity(0x9161105a), revision = AspectContractRevision(1),);
worth_query_field!(
    pub IdentityIdField for WorthUiApplicationSchema, WorthUiRecord, IdentityAspect:
    String => StringApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub QueryTextStatusField for WorthUiApplicationSchema, WorthUiRecord, QueryTextAspect:
    String => StringApplicationValueBinding, read_write, equality
);
worth_query_field!(
    pub QueryRevisionValueField for WorthUiApplicationSchema, WorthUiRecord, QueryRevisionAspect:
    u64 => U64ApplicationValueBinding, read_write, equality
);
worth_query_field!(
    pub CollectionItemStatusField for WorthUiApplicationSchema, WorthUiRecord, CollectionItemAspect:
    String => StringApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub CollectionItemKeyField for WorthUiApplicationSchema, WorthUiRecord, CollectionItemAspect:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub MeasurementValueField for WorthUiApplicationSchema, WorthUiRecord, MeasurementAspect:
    UiMeasurementValue => UiMeasurementValueBinding, read_only, equality
);
worth_query_field!(
    pub SizeValueField for WorthUiApplicationSchema, WorthUiRecord, SizeAspect:
    UiSizeValue => UiSizeValueBinding, read_only, equality
);

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
