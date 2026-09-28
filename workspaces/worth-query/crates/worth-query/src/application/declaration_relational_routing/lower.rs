use crate::application::WorthQueryDeclarationEnvelope;

use super::{
    artifact::WorthQueryDeclarationRelationalBinding,
    contract::{
        WorthQueryDeclarationRelationalAuthorityFamily, WorthQueryDeclarationRelationalTruthClaim,
        WorthQueryDeclarationRelationalTruthContract,
    },
};

pub(crate) fn worth_query_lower_relational_binding<
    D: crate::application::WorthQueryDomainEntryMarker,
    I: crate::application::WorthQueryDeclarationInput<D>,
>(
    _envelope: &WorthQueryDeclarationEnvelope<D, I>,
    contract: WorthQueryDeclarationRelationalTruthContract,
) -> (
    WorthQueryDeclarationRelationalTruthClaim,
    WorthQueryDeclarationRelationalAuthorityFamily,
    WorthQueryDeclarationRelationalBinding,
) {
    let binding = match contract.authority_family() {
        WorthQueryDeclarationRelationalAuthorityFamily::Runtime => {
            WorthQueryDeclarationRelationalBinding::Runtime("worth_relational::facade::runtime")
        }
        WorthQueryDeclarationRelationalAuthorityFamily::History => {
            WorthQueryDeclarationRelationalBinding::History("worth_relational::facade::history")
        }
        WorthQueryDeclarationRelationalAuthorityFamily::GroupedTruth => {
            WorthQueryDeclarationRelationalBinding::GroupedTruth(
                "worth_runtime_bridge::facade::RelationalGroupedProjectionArtifact",
            )
        }
        WorthQueryDeclarationRelationalAuthorityFamily::CommitStrategies => {
            WorthQueryDeclarationRelationalBinding::CommitStrategies(
                "worth_relational::facade::commit_strategies",
            )
        }
        WorthQueryDeclarationRelationalAuthorityFamily::BridgeSource => {
            WorthQueryDeclarationRelationalBinding::BridgeSource(
                "worth_relational::facade::bridge::RuntimeBridgeRelationalSource",
            )
        }
    };
    (contract.truth_claim(), contract.authority_family(), binding)
}
