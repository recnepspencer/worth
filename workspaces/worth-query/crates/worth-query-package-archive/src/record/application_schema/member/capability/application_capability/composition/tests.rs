//! Portable role/source/comparison meaning, without native admission authority.
use super::*;
use crate::binary_output::BinaryOutput;
use crate::limits::WorthQueryPackageArchiveLimits;
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityRecordedActorRequirement as Requirement,
    ApplicationCapabilityRecordedActorRule as Rule,
    ApplicationRecordedActorComparison as Comparison,
    ApplicationWorkflowRecordedActorSource as Source, ApplicationWorkflowRecordedRole as Role,
};

fn composition(rule: Rule) -> ApplicationCapabilityComposition {
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(
                ApplicationCapabilityGraphRule::from_untrusted_parts(
                    WorthQueryPortableApplicationCapabilityGraphRuleParts {
                        requirements: vec![],
                    },
                ),
            ),
            ApplicationCapabilityDenyRule::NotApplicable,
            ApplicationCapabilityConflictRule::NotApplicable,
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::NotApplicable,
            ApplicationCapabilityDistinctActorRule::NotApplicable,
            rule,
        ),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::Forbidden,
            ApplicationCapabilityDisclosureRule::NotApplicable,
        ),
    )
}

#[test]
fn current_composition_preserves_recorded_source_role_and_full_comparison() {
    let mut rules = vec![Rule::not_applicable()];
    for (source, role) in [
        (Source::BoundProposal, Role::Proposer),
        (Source::BoundApproval, Role::Approver),
        (Source::BoundTransition, Role::Advancer),
        (Source::BoundTransition, Role::Executor),
    ] {
        for comparison in [Comparison::SamePrincipal, Comparison::DifferentPrincipal] {
            rules.push(Rule::required(Requirement::new(source, role, comparison)));
        }
    }
    let mut encodings = Vec::new();
    for rule in rules {
        let meaning = composition(rule.clone());
        let mut output = BinaryOutput::with_capacity(128);
        write(&mut output, &meaning).unwrap();
        let bytes = output.into_bytes();
        assert!(encodings.iter().all(|prior| prior != &bytes));
        let mut input = BinaryInput::new(&bytes);
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            bytes.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        let decoded = decode(&mut input, &mut budget).unwrap();
        assert!(input.is_finished());
        assert_eq!(decoded.actors().recorded_actor(), &rule);
        assert_eq!(decoded, meaning);
        encodings.push(bytes);
    }
}
