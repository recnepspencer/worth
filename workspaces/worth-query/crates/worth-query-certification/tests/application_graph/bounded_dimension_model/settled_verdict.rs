//! Naming what one presented request actually settled as.
//!
//! The court asserts verdicts, not shapes. Anything the platform can settle
//! that this court did not name is a failure here, with the platform's own
//! evidence in the panic, so a changed denial cannot pass as an expected one.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome,
};

use super::presented_request::DimensionOutcome;

/// What one set-dimension request settled as on one exact occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DimensionVerdict {
    /// The commit landed, and the action reported writing this dimension.
    Performed(u64),
    /// A commit-boundary rule carrying this exact identity refused the
    /// candidate before it could reach the branch.
    RuleViolated {
        rule: String,
        major: u16,
        minor: u16,
    },
    /// The presented program is not the program this occurrence is running.
    ProgramNotActiveOnOccurrence {
        stage: WorthQueryApplicationCommitDenialStage,
    },
}

impl DimensionVerdict {
    /// The verdict a host reaches when the named rule refuses the candidate.
    pub fn violated(rule: &str) -> Self {
        Self::RuleViolated {
            rule: rule.to_owned(),
            major: 1,
            minor: 0,
        }
    }

    /// The verdict a host reaches when a rostered but inactive program is
    /// presented on this occurrence. Proposal binding is the stage that refuses
    /// it, which is before any effect is lowered.
    pub fn inactive() -> Self {
        Self::ProgramNotActiveOnOccurrence {
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
        }
    }
}

/// Reads one settled request as a verdict, refusing to collapse anything the
/// court did not name.
pub fn settle(
    settlement: Result<DimensionOutcome, WorthQueryApplicationRequestMutationDenial>,
) -> DimensionVerdict {
    let outcome = settlement.expect("the presented request must reach this occurrence");
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { result, .. } => {
            DimensionVerdict::Performed(result.dimension)
        }
        WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationCommitOutcome::Denied(denial),
        ) => match denial.kind() {
            WorthQueryApplicationCommitDenialKind::CustomInvariantDenied => {
                let identity = denial
                    .custom_invariant_violation_identity()
                    .expect("a rule denial must name the rule it violated");
                DimensionVerdict::RuleViolated {
                    rule: identity.rule_id.as_str().to_owned(),
                    major: identity.semantic_version.major,
                    minor: identity.semantic_version.minor,
                }
            }
            WorthQueryApplicationCommitDenialKind::ProgramNotActiveOnOccurrence => {
                DimensionVerdict::ProgramNotActiveOnOccurrence {
                    stage: denial.stage(),
                }
            }
            unexpected => panic!(
                "unexpected commit denial {unexpected:?} at {:?}: {:?}",
                denial.stage(),
                denial.detail()
            ),
        },
        unexpected => panic!("unexpected settlement: {unexpected:?}"),
    }
}
