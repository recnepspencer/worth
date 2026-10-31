//! One policy pass precedes binding to an actual read subject.
use super::*;

/// Internal unbound policy data; no session, read proof or permission.
pub(in crate::domain_computation) struct WorthQueryPreparedApplicationGovernance {
    decision: PreparedDecision,
}

enum PreparedDecision {
    Public,
    Governed {
        pending: WorthQueryPendingApplicationQueryGovernance,
        internal_field_rules: GovernedInternalFieldRules,
        disclosure: WorthQueryApplicationDisclosureDecision,
    },
}

pub(super) fn prepare(
    contract: WorthQueryAdmittedApplicationDisclosureContract,
    pending: Option<WorthQueryPendingApplicationQueryGovernance>,
) -> Result<WorthQueryPreparedApplicationGovernance, WorthQueryApplicationQueryGovernanceDenialKind>
{
    match (contract, pending) {
        (WorthQueryAdmittedApplicationDisclosureContract::Public, None) => {
            Ok(WorthQueryPreparedApplicationGovernance {
                decision: PreparedDecision::Public,
            })
        }
        (
            WorthQueryAdmittedApplicationDisclosureContract::Governed {
                classification,
                capability_name,
                capability_type,
                result_rules,
                internal_rules,
                internal_field_rules,
            },
            Some(pending),
        ) => {
            if capability_name != pending.capability_name
                || capability_type != pending.capability_type
            {
                return Err(WorthQueryApplicationQueryGovernanceDenialKind::CapabilityMismatch);
            }
            if internal_rules
                .iter()
                .any(|rule| rule.disclosure_value() != &pending.disclosure_value)
            {
                return Err(
                    WorthQueryApplicationQueryGovernanceDenialKind::InternalComputationDenied,
                );
            }
            let (disclosed, omitted) = result_rules.into_iter().fold(
                (BTreeMap::new(), BTreeMap::new()),
                |(mut disclosed, mut omitted), (slot, rule)| {
                    let target = if rule.disclosure_value() == &pending.disclosure_value {
                        &mut disclosed
                    } else {
                        &mut omitted
                    };
                    target.insert(slot, rule);
                    (disclosed, omitted)
                },
            );
            Ok(WorthQueryPreparedApplicationGovernance {
                decision: PreparedDecision::Governed {
                    pending,
                    internal_field_rules,
                    disclosure: WorthQueryApplicationDisclosureDecision {
                        classification,
                        disclosed,
                        omitted,
                    },
                },
            })
        }
        _ => Err(WorthQueryApplicationQueryGovernanceDenialKind::Required),
    }
}

impl WorthQueryPreparedApplicationGovernance {
    pub(super) fn bind(
        self,
        binding: WorthQueryApplicationGovernanceBinding,
    ) -> WorthQueryApplicationQueryGovernance {
        match self.decision {
            PreparedDecision::Public => WorthQueryApplicationQueryGovernance::Public,
            PreparedDecision::Governed {
                pending,
                internal_field_rules,
                disclosure,
            } => WorthQueryApplicationQueryGovernance::Governed {
                capability_name: pending.capability_name,
                capability_type: pending.capability_type,
                disclosure_value: pending.disclosure_value.clone(),
                authorization_canonical_work: pending.authorization_canonical_work,
                computation: WorthQueryApplicationInternalComputationAuthority {
                    binding,
                    internal_field_rules,
                    disclosure_value: pending.disclosure_value,
                },
                disclosure,
                authorization: pending.authorization,
            },
        }
    }

    pub(in crate::domain_computation) fn is_disclosed(
        &self,
        slot: &ApplicationQueryResultSlotKey,
    ) -> bool {
        match &self.decision {
            PreparedDecision::Public => true,
            PreparedDecision::Governed { disclosure, .. } => {
                disclosure.disclosed.contains_key(slot)
            }
        }
    }

    pub(in crate::domain_computation) fn authorization_mut(
        &mut self,
    ) -> Option<&mut WorthQueryRetainedCapabilityAuthorization> {
        match &mut self.decision {
            PreparedDecision::Public => None,
            PreparedDecision::Governed { pending, .. } => Some(&mut pending.authorization),
        }
    }
}
