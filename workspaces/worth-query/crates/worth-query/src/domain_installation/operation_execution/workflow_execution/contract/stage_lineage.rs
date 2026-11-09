use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowStageLineageDenial {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    RuntimeMutationEvidenceRequired,
    AuthoritativeContinuityEvidenceRequired,
    IdentityEvolutionAdmission(crate::identity_evolution::IdentityEvolutionAdmissionError),
    IdentityEvolutionOutcomeMismatch,
}

pub(super) fn lineage_traversal(
    mutation_receipt: &crate::runtime::WorthQueryWriteReceipt,
) -> Result<
    (
        crate::identity_evolution::LineageTraversalDescriptor,
        Option<crate::memory_workspace::WorthQueryEntityIdentity>,
    ),
    WorthQueryWorkflowStageLineageDenial,
> {
    let required_target = || {
        mutation_receipt
            .target_entity_identity()
            .ok_or(WorthQueryWorkflowStageLineageDenial::AuthoritativeContinuityEvidenceRequired)
    };
    match mutation_receipt.mutation_family() {
        crate::runtime::WorthQueryMutationFamily::Insert => {
            let target = required_target()?;
            Ok((
                crate::identity_evolution::LineageTraversalDescriptor::generated_identity(
                    target.evidence_identity().as_str().to_owned(),
                ),
                Some(target.clone()),
            ))
        }
        crate::runtime::WorthQueryMutationFamily::Delete => {
            let target = required_target()?;
            Ok((
                crate::identity_evolution::LineageTraversalDescriptor::retired_identity(
                    target.evidence_identity().as_str().to_owned(),
                ),
                Some(target.clone()),
            ))
        }
        crate::runtime::WorthQueryMutationFamily::Update => Ok((
            authoritative_continuity_descriptor(
                mutation_receipt.continuity_mutation_evidence().ok_or(
                    WorthQueryWorkflowStageLineageDenial::AuthoritativeContinuityEvidenceRequired,
                )?,
            )?,
            None,
        )),
        crate::runtime::WorthQueryMutationFamily::Assertion => {
            Err(WorthQueryWorkflowStageLineageDenial::AuthoritativeContinuityEvidenceRequired)
        }
    }
}

fn authoritative_continuity_descriptor(
    continuity: &crate::runtime::WorthQueryContinuityMutationEvidence,
) -> Result<
    crate::identity_evolution::LineageTraversalDescriptor,
    WorthQueryWorkflowStageLineageDenial,
> {
    use crate::runtime::WorthQueryContinuityOutcomeClass as Outcome;

    let anchor = continuity
        .prior_authoritative_identity()
        .evidence_identity()
        .as_str()
        .to_owned();
    let successors = continuity
        .successor_authoritative_identities()
        .iter()
        .map(|identity| identity.evidence_identity().as_str().to_owned())
        .collect::<Vec<_>>();
    match continuity.outcome_class() {
        Outcome::ContinuesAsSingleSuccessor => Ok(
            crate::identity_evolution::LineageTraversalDescriptor::direct_successor_exact(
                anchor,
                successors[0].clone(),
            ),
        ),
        Outcome::ContinuesAsSplitSuccessors => Ok(
            crate::identity_evolution::LineageTraversalDescriptor::direct_split_successors_exact(
                anchor, successors,
            ),
        ),
        Outcome::ContinuesViaTruthLoweredCanonicalMergeSuccessor => Ok(
            crate::identity_evolution::LineageTraversalDescriptor::direct_merge_successor_exact(
                anchor,
                successors[0].clone(),
            ),
        ),
        Outcome::RejectedNoAuthoritativeSuccessor
        | Outcome::RejectedAmbiguousSuccessor
        | Outcome::RejectedUnsupportedContinuityClass
        | Outcome::RejectedHistoricalResolutionFailure => {
            Err(WorthQueryWorkflowStageLineageDenial::AuthoritativeContinuityEvidenceRequired)
        }
    }
}

impl From<IdentityDerivationDenial> for WorthQueryWorkflowStageLineageDenial {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}
