use axum::body::Bytes;

use bank_http_adapter::{
    BankHttpAccountActivity, BankHttpAccountActivityEvent, BankHttpQueryBasis,
    BankHttpQueryBasisPosture, BankHttpQueryCapabilityPurpose, BankHttpQueryDisclosure,
    BankHttpQueryDisclosurePosture, BankHttpQueryOmissionPosture, BankHttpQueryPublication,
};

use super::typed_event;

#[test]
fn typed_update_uses_the_current_exact_basis_shape() {
    let event = update_event("basis-shape");
    let frame = std::str::from_utf8(&event).expect("typed update frame is UTF-8");
    let data = frame
        .lines()
        .find_map(|line| line.strip_prefix("data: "))
        .expect("typed update frame carries JSON data");
    let value: serde_json::Value = serde_json::from_str(data).expect("typed update JSON");
    let basis = &value["publication"]["basis"];

    assert_eq!(basis["snapshot"], 1);
    assert_eq!(basis["version"], 1);
    assert!(basis.get("lease").is_none());
}

pub(super) fn update_event(request_id: &str) -> Bytes {
    typed_event(BankHttpAccountActivityEvent::Update {
        request_id: request_id.to_owned(),
        activity: BankHttpAccountActivity {
            account: "fixture:100".to_owned(),
            entries: Vec::new(),
        },
        publication: BankHttpQueryPublication {
            query_identity: "query".to_owned(),
            parameter_binding_identity: "binding".to_owned(),
            basis: BankHttpQueryBasis {
                runtime_instance: 1,
                branch: "ordinary".to_owned(),
                snapshot: 1,
                version: 1,
                posture: BankHttpQueryBasisPosture::SelectedProduct,
            },
            capability_purpose: BankHttpQueryCapabilityPurpose::AccountActivityReview,
            disclosure: BankHttpQueryDisclosure {
                posture: BankHttpQueryDisclosurePosture::Public,
                omission: BankHttpQueryOmissionPosture::NoOmission,
                decision_count: 0,
                disclosed_value_count: 0,
                omitted_value_count: 0,
                authorization_decision_fact_count: 0,
            },
        },
    })
}
