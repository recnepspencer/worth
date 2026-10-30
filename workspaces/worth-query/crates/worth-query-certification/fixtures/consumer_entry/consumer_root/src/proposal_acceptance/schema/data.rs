//! Stored intent is canonical domain data, with no operational custody.
use worth_query_decl::facade::application_schema::WorthValue;
use worth_query_decl::facade::worth_query_value_binding;

#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub struct Coordinates {
    pub partition: u32,
    pub slot: u64,
    pub generation: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub struct ModelReference {
    pub label: String,
    pub runtime: u64,
    pub occurrence: Coordinates,
}

#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub struct TargetReference {
    pub label: String,
    pub occurrence: Coordinates,
}

#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub struct RetentionEdit {
    pub model: ModelReference,
    pub target: TargetReference,
    pub days: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub enum AuthoredIntent {
    Retention(RetentionEdit),
}

#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub struct StoredProposal {
    pub format_version: u16,
    pub revision: u64,
    pub model: ModelReference,
    pub expected_basis: String,
    pub intents: Vec<AuthoredIntent>,
}

worth_query_value_binding!(pub StoredProposalBinding for StoredProposal {
    identity: "proposal-application.stored-proposal.v1", scalar: String, canonical,
});
