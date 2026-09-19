use worth_query_host::facade::publication::domain_computation::{
    WorthQueryApplicationQueryPublicationReceipt, WorthQueryPublishedApplicationBasisPosture,
    WorthQueryPublishedApplicationDisclosurePosture,
    WorthQueryPublishedApplicationQueryOmissionPosture,
};

use super::super::protocol::{
    BankHttpQueryBasis, BankHttpQueryBasisPosture, BankHttpQueryCapabilityPurpose,
    BankHttpQueryDisclosure, BankHttpQueryDisclosurePosture, BankHttpQueryOmissionPosture,
    BankHttpQueryPublication,
};

pub(super) fn describe_query_publication(
    receipt: &WorthQueryApplicationQueryPublicationReceipt,
    capability_purpose: BankHttpQueryCapabilityPurpose,
) -> BankHttpQueryPublication {
    let inspection = receipt.inspect();
    let basis = inspection.basis();
    let disclosure = receipt.disclosure();
    BankHttpQueryPublication {
        query_identity: inspection.query_identity().to_owned(),
        parameter_binding_identity: inspection.parameter_binding_identity().to_owned(),
        basis: BankHttpQueryBasis {
            runtime_instance: basis.runtime_instance(),
            branch: basis.branch().to_owned(),
            snapshot: basis.snapshot(),
            version: basis.version(),
            posture: match basis.posture() {
                WorthQueryPublishedApplicationBasisPosture::SelectedProduct => {
                    BankHttpQueryBasisPosture::SelectedProduct
                }
            },
        },
        capability_purpose,
        disclosure: BankHttpQueryDisclosure {
            posture: match disclosure.posture() {
                WorthQueryPublishedApplicationDisclosurePosture::Public => {
                    BankHttpQueryDisclosurePosture::Public
                }
                WorthQueryPublishedApplicationDisclosurePosture::Governed => {
                    BankHttpQueryDisclosurePosture::Governed
                }
            },
            omission: match inspection.omission_posture() {
                WorthQueryPublishedApplicationQueryOmissionPosture::NoOmission => {
                    BankHttpQueryOmissionPosture::NoOmission
                }
                WorthQueryPublishedApplicationQueryOmissionPosture::GovernedOmission => {
                    BankHttpQueryOmissionPosture::GovernedOmission
                }
            },
            decision_count: disclosure.disclosure_decision_count(),
            disclosed_value_count: disclosure.disclosed_value_count(),
            omitted_value_count: disclosure.omitted_value_count(),
            authorization_decision_fact_count: disclosure.authorization_decision_fact_count(),
        },
    }
}
