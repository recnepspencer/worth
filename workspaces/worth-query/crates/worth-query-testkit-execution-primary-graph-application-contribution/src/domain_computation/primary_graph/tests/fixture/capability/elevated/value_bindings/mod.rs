use worth_foundational::facade::AspectValue;
use worth_foundational::facade::InternedString;
use worth_foundational::facade::ScalarAspectType;
use worth_query_declaration::facade::application_schema::ApplicationIdentityScalarValueBinding;
use worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
use worth_query_declaration::facade::application_schema::ApplicationValueDecodeAvailable;
use worth_query_declaration::facade::application_schema::ApplicationValueDecodeDenial;
use worth_query_declaration::facade::application_schema::ApplicationValueEncodeDenial;
use worth_query_declaration::facade::application_schema::ApplicationValueIsIdentity;
use worth_query_declaration::facade::application_schema::ApplicationValueSignedAggregateUnavailable;
use worth_query_declaration::facade::application_schema::ApplicationValueValidationDenial;

use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationStatus;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewKind;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewStatus;

pub fn encode_elevation_status(value: &CapabilityElevationStatus) -> InternedString {
    InternedString::from(match value {
        CapabilityElevationStatus::Requested => "requested",
        CapabilityElevationStatus::Approved => "approved",
        CapabilityElevationStatus::Expired => "expired",
        CapabilityElevationStatus::Revoked => "revoked",
    })
}

pub fn decode_elevation_status(value: InternedString) -> Option<CapabilityElevationStatus> {
    let InternedString::Raw(value) = value else {
        return None;
    };
    match value.as_str() {
        "requested" => Some(CapabilityElevationStatus::Requested),
        "approved" => Some(CapabilityElevationStatus::Approved),
        "expired" => Some(CapabilityElevationStatus::Expired),
        "revoked" => Some(CapabilityElevationStatus::Revoked),
        _ => None,
    }
}

worth_query_declaration::worth_query_value_binding! {
    pub CapabilityElevationStatusBinding for CapabilityElevationStatus {
        identity: "worth.query.test.execution.capability.elevation_status.v1",
        scalar: String,
        encode: encode_elevation_status,
        decode: decode_elevation_status,
    }
}

pub fn encode_review_status(value: &CapabilityReviewStatus) -> InternedString {
    InternedString::from(match value {
        CapabilityReviewStatus::Required => "required",
        CapabilityReviewStatus::Completed => "completed",
    })
}

pub fn decode_review_status(value: InternedString) -> Option<CapabilityReviewStatus> {
    let InternedString::Raw(value) = value else {
        return None;
    };
    match value.as_str() {
        "required" => Some(CapabilityReviewStatus::Required),
        "completed" => Some(CapabilityReviewStatus::Completed),
        _ => None,
    }
}

worth_query_declaration::worth_query_value_binding! {
    pub CapabilityReviewStatusBinding for CapabilityReviewStatus {
        identity: "worth.query.test.execution.capability.review_status.v1",
        scalar: String,
        encode: encode_review_status,
        decode: decode_review_status,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityReviewKindBinding;

impl ApplicationScalarValueBinding for CapabilityReviewKindBinding {
    type Value = CapabilityReviewKind;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;

    const IDENTITY_NAME: &'static str = "worth.query.test.execution.capability.review_kind.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;

    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }

    fn encode(_: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::String(InternedString::from("elevation")))
    }
}

impl ApplicationReadableScalarValueBinding for CapabilityReviewKindBinding {
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        match value {
            AspectValue::String(text) if text.raw_text() == Some("elevation") => {
                Ok(CapabilityReviewKind::Elevation)
            }
            AspectValue::String(_) => Err(ApplicationValueDecodeDenial::CodecRejected {
                binding_identity: Self::IDENTITY,
            }),
            value => Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
                binding_identity: Self::IDENTITY,
                expected: Self::SCALAR_FAMILY,
                observed: value.value_family(),
            }),
        }
    }
}

impl ApplicationIdentityScalarValueBinding for CapabilityReviewKindBinding {}
