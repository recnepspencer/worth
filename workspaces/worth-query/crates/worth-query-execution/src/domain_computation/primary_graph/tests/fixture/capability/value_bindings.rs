use super::{CapabilityAction, CapabilityDisclosure, CapabilityPurpose, CapabilityStatus};
use worth_foundational::facade::InternedString;

fn encode_capability_action(value: &CapabilityAction) -> InternedString {
    InternedString::from(match value {
        CapabilityAction::Touch => "touch",
        CapabilityAction::Inspect => "inspect",
        CapabilityAction::Disburse => "disburse",
        CapabilityAction::RequestElevation => "request-elevation",
        CapabilityAction::ApproveElevation => "approve-elevation",
        CapabilityAction::RevokeElevation => "revoke-elevation",
        CapabilityAction::CompleteReview => "complete-review",
    })
}

fn decode_capability_action(value: InternedString) -> Option<CapabilityAction> {
    let InternedString::Raw(value) = value else {
        return None;
    };
    match value.as_str() {
        "touch" => Some(CapabilityAction::Touch),
        "inspect" => Some(CapabilityAction::Inspect),
        "disburse" => Some(CapabilityAction::Disburse),
        "request-elevation" => Some(CapabilityAction::RequestElevation),
        "approve-elevation" => Some(CapabilityAction::ApproveElevation),
        "revoke-elevation" => Some(CapabilityAction::RevokeElevation),
        "complete-review" => Some(CapabilityAction::CompleteReview),
        _ => None,
    }
}

worth_query_declaration::worth_query_value_binding! {
    pub CapabilityActionBinding for CapabilityAction {
        identity: "worth.query.test.execution.capability.action.v1",
        scalar: String,
        encode: encode_capability_action,
        decode: decode_capability_action,
    }
}

fn encode_capability_purpose(value: &CapabilityPurpose) -> InternedString {
    InternedString::from(match value {
        CapabilityPurpose::AccountMaintenance => "account-maintenance",
        CapabilityPurpose::Audit => "audit",
    })
}

fn decode_capability_purpose(value: InternedString) -> Option<CapabilityPurpose> {
    let InternedString::Raw(value) = value else {
        return None;
    };
    match value.as_str() {
        "account-maintenance" => Some(CapabilityPurpose::AccountMaintenance),
        "audit" => Some(CapabilityPurpose::Audit),
        _ => None,
    }
}

worth_query_declaration::worth_query_value_binding! {
    pub CapabilityPurposeBinding for CapabilityPurpose {
        identity: "worth.query.test.execution.capability.purpose.v1",
        scalar: String,
        encode: encode_capability_purpose,
        decode: decode_capability_purpose,
    }
}

fn encode_capability_status(value: &CapabilityStatus) -> InternedString {
    InternedString::from(match value {
        CapabilityStatus::Active => "active",
        CapabilityStatus::Revoked => "revoked",
    })
}

fn decode_capability_status(value: InternedString) -> Option<CapabilityStatus> {
    let InternedString::Raw(value) = value else {
        return None;
    };
    match value.as_str() {
        "active" => Some(CapabilityStatus::Active),
        "revoked" => Some(CapabilityStatus::Revoked),
        _ => None,
    }
}

worth_query_declaration::worth_query_value_binding! {
    pub CapabilityStatusBinding for CapabilityStatus {
        identity: "worth.query.test.execution.capability.status.v1",
        scalar: String,
        encode: encode_capability_status,
        decode: decode_capability_status,
    }
}

fn encode_capability_disclosure(value: &CapabilityDisclosure) -> InternedString {
    InternedString::from(match value {
        CapabilityDisclosure::AccountActivity => "account-activity",
        CapabilityDisclosure::PrivateLabel => "private-label",
    })
}

fn decode_capability_disclosure(value: InternedString) -> Option<CapabilityDisclosure> {
    let InternedString::Raw(value) = value else {
        return None;
    };
    match value.as_str() {
        "account-activity" => Some(CapabilityDisclosure::AccountActivity),
        "private-label" => Some(CapabilityDisclosure::PrivateLabel),
        _ => None,
    }
}

worth_query_declaration::worth_query_value_binding! {
    pub CapabilityDisclosureBinding for CapabilityDisclosure {
        identity: "worth.query.test.execution.capability.disclosure.v1",
        scalar: String,
        encode: encode_capability_disclosure,
        decode: decode_capability_disclosure,
    }
}
