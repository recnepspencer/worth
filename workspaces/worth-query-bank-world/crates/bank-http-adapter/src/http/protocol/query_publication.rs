use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpQueryCapabilityPurpose {
    AccountServicing,
    AccountActivityReview,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpQueryBasisPosture {
    SelectedProduct,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpQueryBasis {
    pub runtime_instance: u64,
    pub branch: String,
    pub snapshot: u64,
    pub version: u64,
    pub posture: BankHttpQueryBasisPosture,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpQueryDisclosurePosture {
    Public,
    Governed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpQueryOmissionPosture {
    NoOmission,
    GovernedOmission,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BankHttpQueryDisclosure {
    pub posture: BankHttpQueryDisclosurePosture,
    pub omission: BankHttpQueryOmissionPosture,
    pub decision_count: usize,
    pub disclosed_value_count: usize,
    pub omitted_value_count: usize,
    pub authorization_decision_fact_count: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BankHttpQueryPublication {
    pub query_identity: String,
    pub parameter_binding_identity: String,
    pub basis: BankHttpQueryBasis,
    pub capability_purpose: BankHttpQueryCapabilityPurpose,
    pub disclosure: BankHttpQueryDisclosure,
}
