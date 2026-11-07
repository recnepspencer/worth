//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
use worth_query_decl::facade::authentication::*;
pub struct UiRecords;
impl fragment::Fragment for UiRecords {
    const NAME: &'static str = "ui_records";
    type Bindings = UiRecordsBindings;
    type Members = (
        WorthUiExternalPrincipalMapping,
        WorthUiPrincipal,
        WorthUiRecord,
    );
}
pub struct UiRecordsBindings;
impl fragment::BindingSet for UiRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for UiRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for UiRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for UiRecordsBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<String> for UiRecordsBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<UiMeasurementValue> for UiRecordsBindings {
    type Binding = UiMeasurementValueBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<UiSizeValue> for UiRecordsBindings {
    type Binding = UiSizeValueBinding;

    const WRITABLE: bool = false;
}
pub struct WorthUiExternalPrincipalMapping;
impl fragment::Entity for WorthUiExternalPrincipalMapping {
    const NAME: &'static str = "WorthUiExternalPrincipalMapping";
    type Aspects = (WorthUiExternalIdentity,);
    type Relations = (WorthUiExternalPrincipal,);
}
impl WorthUiExternalPrincipalMapping {
    pub fn reference() -> ApplicationEntityRef<WorthUiApplicationSchema, Self> {
        entity(UiRecords, Self)
    }
}
pub struct WorthUiPrincipal;
impl fragment::Entity for WorthUiPrincipal {
    const NAME: &'static str = "WorthUiPrincipal";
    type Aspects = (WorthUiPrincipalIdentity,);
    type Relations = ();
}
impl WorthUiPrincipal {
    pub fn reference() -> ApplicationEntityRef<WorthUiApplicationSchema, Self> {
        entity(UiRecords, Self)
    }
}
pub struct WorthUiRecord;
impl fragment::Entity for WorthUiRecord {
    const NAME: &'static str = "WorthUiRecord";
    type Aspects = (
        IdentityAspect,
        QueryTextAspect,
        QueryRevisionAspect,
        CollectionItemAspect,
        MeasurementAspect,
        SizeAspect,
    );
    type Relations = ();
}
impl WorthUiRecord {
    pub fn reference() -> ApplicationEntityRef<WorthUiApplicationSchema, Self> {
        entity(UiRecords, Self)
    }
}
pub struct WorthUiExternalIdentity;
impl fragment::Aspect for WorthUiExternalIdentity {
    const NAME: &'static str = "WorthUiExternalIdentity";
    type Row = WorthUiExternalIdentityValue;
}
impl WorthUiExternalIdentity {
    pub fn reference() -> AspectSelection<UiRecords, WorthUiExternalPrincipalMapping, Self> {
        aspect(UiRecords, WorthUiExternalPrincipalMapping, Self)
    }
}
pub struct WorthUiExternalIdentityValue {
    pub worth_ui_external_identity_key: WorthQueryExternalPrincipalIdentity,
    pub worth_ui_mapping_status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<WorthUiExternalIdentityValue> for UiRecordsBindings {
    fn record() -> impl fragment::Shape<WorthUiExternalIdentityValue, Self> {
        fragment::field(
            "WorthUiExternalIdentityKey",
            |row: &WorthUiExternalIdentityValue| row.worth_ui_external_identity_key.clone(),
        )
        .equality_queryable(true)
        .and(
            "WorthUiMappingStatus",
            |row: &WorthUiExternalIdentityValue| row.worth_ui_mapping_status,
        )
        .equality_queryable(true)
        .build(
            |worth_ui_external_identity_key,
             worth_ui_mapping_status,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(WorthUiExternalIdentityValue {
                    worth_ui_external_identity_key,
                    worth_ui_mapping_status,
                })
            },
        )
    }
}
pub struct WorthUiPrincipalIdentity;
impl fragment::Aspect for WorthUiPrincipalIdentity {
    const NAME: &'static str = "WorthUiPrincipalIdentity";
    type Row = WorthUiPrincipalIdentityValue;
}
impl WorthUiPrincipalIdentity {
    pub fn reference() -> AspectSelection<UiRecords, WorthUiPrincipal, Self> {
        aspect(UiRecords, WorthUiPrincipal, Self)
    }
}
pub struct WorthUiPrincipalIdentityValue {
    pub worth_ui_principal_id: u64,
}
impl fragment::Record<WorthUiPrincipalIdentityValue> for UiRecordsBindings {
    fn record() -> impl fragment::Shape<WorthUiPrincipalIdentityValue, Self> {
        fragment::field(
            "WorthUiPrincipalId",
            |row: &WorthUiPrincipalIdentityValue| row.worth_ui_principal_id,
        )
        .equality_queryable(true)
        .read_only()
        .build(|worth_ui_principal_id, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(WorthUiPrincipalIdentityValue {
                worth_ui_principal_id,
            })
        })
    }
}
pub struct IdentityAspect;
impl fragment::Aspect for IdentityAspect {
    const NAME: &'static str = "IdentityAspect";
    type Row = IdentityAspectValue;
}
impl IdentityAspect {
    pub fn reference() -> AspectSelection<UiRecords, WorthUiRecord, Self> {
        aspect(UiRecords, WorthUiRecord, Self)
    }
}
pub struct IdentityAspectValue {
    pub identity_id_field: String,
}
impl fragment::Record<IdentityAspectValue> for UiRecordsBindings {
    fn record() -> impl fragment::Shape<IdentityAspectValue, Self> {
        fragment::field("IdentityIdField", |row: &IdentityAspectValue| {
            row.identity_id_field.clone()
        })
        .equality_queryable(true)
        .read_only()
        .build(|identity_id_field, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(IdentityAspectValue { identity_id_field })
        })
    }
}
pub struct QueryTextAspect;
impl fragment::Aspect for QueryTextAspect {
    const NAME: &'static str = "QueryTextAspect";
    type Row = QueryTextAspectValue;
}
impl QueryTextAspect {
    pub fn reference() -> AspectSelection<UiRecords, WorthUiRecord, Self> {
        aspect(UiRecords, WorthUiRecord, Self)
    }
}
pub struct QueryTextAspectValue {
    pub query_text_status_field: String,
}
impl fragment::Record<QueryTextAspectValue> for UiRecordsBindings {
    fn record() -> impl fragment::Shape<QueryTextAspectValue, Self> {
        fragment::field("QueryTextStatusField", |row: &QueryTextAspectValue| {
            row.query_text_status_field.clone()
        })
        .equality_queryable(true)
        .build(
            |query_text_status_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(QueryTextAspectValue {
                    query_text_status_field,
                })
            },
        )
    }
}
pub struct QueryRevisionAspect;
impl fragment::Aspect for QueryRevisionAspect {
    const NAME: &'static str = "QueryRevisionAspect";
    type Row = QueryRevisionAspectValue;
}
impl QueryRevisionAspect {
    pub fn reference() -> AspectSelection<UiRecords, WorthUiRecord, Self> {
        aspect(UiRecords, WorthUiRecord, Self)
    }
}
pub struct QueryRevisionAspectValue {
    pub query_revision_value_field: u64,
}
impl fragment::Record<QueryRevisionAspectValue> for UiRecordsBindings {
    fn record() -> impl fragment::Shape<QueryRevisionAspectValue, Self> {
        fragment::field(
            "QueryRevisionValueField",
            |row: &QueryRevisionAspectValue| row.query_revision_value_field,
        )
        .equality_queryable(true)
        .build(
            |query_revision_value_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(QueryRevisionAspectValue {
                    query_revision_value_field,
                })
            },
        )
    }
}
pub struct CollectionItemAspect;
impl fragment::Aspect for CollectionItemAspect {
    const NAME: &'static str = "CollectionItemAspect";
    type Row = CollectionItemAspectValue;
}
impl CollectionItemAspect {
    pub fn reference() -> AspectSelection<UiRecords, WorthUiRecord, Self> {
        aspect(UiRecords, WorthUiRecord, Self)
    }
}
pub struct CollectionItemAspectValue {
    pub collection_item_status_field: String,
    pub collection_item_key_field: u64,
}
impl fragment::Record<CollectionItemAspectValue> for UiRecordsBindings {
    fn record() -> impl fragment::Shape<CollectionItemAspectValue, Self> {
        fragment::field(
            "CollectionItemStatusField",
            |row: &CollectionItemAspectValue| row.collection_item_status_field.clone(),
        )
        .equality_queryable(true)
        .read_only()
        .and(
            "CollectionItemKeyField",
            |row: &CollectionItemAspectValue| row.collection_item_key_field,
        )
        .equality_queryable(true)
        .read_only()
        .build(
            |collection_item_status_field,
             collection_item_key_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CollectionItemAspectValue {
                    collection_item_status_field,
                    collection_item_key_field,
                })
            },
        )
    }
}
pub struct MeasurementAspect;
impl fragment::Aspect for MeasurementAspect {
    const NAME: &'static str = "MeasurementAspect";
    type Row = MeasurementAspectValue;
}
impl MeasurementAspect {
    pub fn reference() -> AspectSelection<UiRecords, WorthUiRecord, Self> {
        aspect(UiRecords, WorthUiRecord, Self)
    }
}
pub struct MeasurementAspectValue {
    pub measurement_value_field: UiMeasurementValue,
}
impl fragment::Record<MeasurementAspectValue> for UiRecordsBindings {
    fn record() -> impl fragment::Shape<MeasurementAspectValue, Self> {
        fragment::field("MeasurementValueField", |row: &MeasurementAspectValue| {
            row.measurement_value_field
        })
        .equality_queryable(true)
        .build(
            |measurement_value_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MeasurementAspectValue {
                    measurement_value_field,
                })
            },
        )
    }
}
pub struct SizeAspect;
impl fragment::Aspect for SizeAspect {
    const NAME: &'static str = "SizeAspect";
    type Row = SizeAspectValue;
}
impl SizeAspect {
    pub fn reference() -> AspectSelection<UiRecords, WorthUiRecord, Self> {
        aspect(UiRecords, WorthUiRecord, Self)
    }
}
pub struct SizeAspectValue {
    pub size_value_field: UiSizeValue,
}
impl fragment::Record<SizeAspectValue> for UiRecordsBindings {
    fn record() -> impl fragment::Shape<SizeAspectValue, Self> {
        fragment::field("SizeValueField", |row: &SizeAspectValue| {
            row.size_value_field
        })
        .equality_queryable(true)
        .build(|size_value_field, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(SizeAspectValue { size_value_field })
        })
    }
}
pub type WorthUiExternalIdentityKey = FieldSelection<
    UiRecords,
    WorthUiExternalPrincipalMapping,
    WorthUiExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type WorthUiMappingStatus = FieldSelection<
    UiRecords,
    WorthUiExternalPrincipalMapping,
    WorthUiExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type WorthUiPrincipalId =
    FieldSelection<UiRecords, WorthUiPrincipal, WorthUiPrincipalIdentity, u64>;
pub type IdentityIdField = FieldSelection<UiRecords, WorthUiRecord, IdentityAspect, String>;
pub type QueryTextStatusField = FieldSelection<UiRecords, WorthUiRecord, QueryTextAspect, String>;
pub type QueryRevisionValueField =
    FieldSelection<UiRecords, WorthUiRecord, QueryRevisionAspect, u64>;
pub type CollectionItemStatusField =
    FieldSelection<UiRecords, WorthUiRecord, CollectionItemAspect, String>;
pub type CollectionItemKeyField =
    FieldSelection<UiRecords, WorthUiRecord, CollectionItemAspect, u64>;
pub type MeasurementValueField =
    FieldSelection<UiRecords, WorthUiRecord, MeasurementAspect, UiMeasurementValue>;
pub type SizeValueField = FieldSelection<UiRecords, WorthUiRecord, SizeAspect, UiSizeValue>;
pub struct WorthUiExternalPrincipal;
impl fragment::Relation for WorthUiExternalPrincipal {
    const NAME: &'static str = "WorthUiExternalPrincipal";
    type To = WorthUiPrincipal;
}
impl WorthUiExternalPrincipal {
    pub fn reference() -> ApplicationRelationRef<
        WorthUiApplicationSchema,
        Self,
        WorthUiExternalPrincipalMapping,
        WorthUiPrincipal,
    > {
        relation(UiRecords, WorthUiExternalPrincipalMapping, Self)
    }
}
