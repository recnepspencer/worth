use super::{
    UiAllocationCadenceBudget, UiAllocationCadenceKind, UiAllocationCommitTarget,
    UiAllocationEvidenceCadence, UiAllocationPartialSettlementLaw, UiAllocationStreamCollapseLaw,
    UiAllocationStreamFamily,
};

mod accessors;
mod pair_policy;
mod policy_join;

use pair_policy::{pair_contract, UiAllocationFamilyPairContract};
use policy_join::{join_contract_policies, resolved_family_policy};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationStreamCompositionDenial {
    EmptyFrame,
    InputBudgetExceeded {
        admitted: u16,
        allowed: u16,
    },
    IllegalFamilyPair {
        left: UiAllocationStreamFamily,
        right: UiAllocationStreamFamily,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationFamilyPairOutcome {
    Compose,
    CoSelect,
    Deny,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAllocationIntermediatePolicyVerdict {
    left: UiAllocationStreamFamily,
    right: UiAllocationStreamFamily,
    outcome: UiAllocationFamilyPairOutcome,
    resolved: UiResolvedAllocationStreamPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiResolvedAllocationPolicyBranch {
    families: Box<[UiAllocationStreamFamily]>,
    policy: UiResolvedAllocationStreamPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAllocationStreamCompositionCounters {
    admitted_family_count: u8,
    admitted_input_count: u16,
    pair_contract_evaluations: u8,
    pair_policy_joins: u8,
    n_way_policy_joins: u8,
    branch_policy_joins: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiResolvedAllocationStreamPolicy {
    commit_lane: UiAllocationResolvedCommitLane,
    target: UiAllocationCommitTarget,
    cadence: UiAllocationCadenceKind,
    budget: UiAllocationCadenceBudget,
    evidence_cadence: UiAllocationEvidenceCadence,
    collapse_law: UiAllocationStreamCollapseLaw,
    partial_settlement_law: UiAllocationPartialSettlementLaw,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAllocationResolvedCommitLane {
    Ordinary,
    ViewportDerived,
    ResizePreview,
    DurableResize,
    DragResize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAllocationStreamCompositionReceipt {
    families: Box<[UiAllocationStreamFamily]>,
    intermediate: Box<[UiAllocationIntermediatePolicyVerdict]>,
    branches: Box<[UiResolvedAllocationPolicyBranch]>,
    policy: UiResolvedAllocationStreamPolicy,
    counters: UiAllocationStreamCompositionCounters,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiAllocationStreamCommitDecision {
    Commit(UiAllocationStreamCompositionReceipt),
    Preview(UiAllocationStreamCompositionReceipt),
    Denied(UiAllocationStreamCompositionDenial),
}

pub(crate) fn replacement_activation_policy(
    catalog_cardinality: u16,
) -> UiResolvedAllocationStreamPolicy {
    UiResolvedAllocationStreamPolicy {
        commit_lane: UiAllocationResolvedCommitLane::Ordinary,
        target: UiAllocationCommitTarget::AllocationOnly,
        cadence: UiAllocationCadenceKind::Terminal,
        budget: UiAllocationCadenceBudget::contract(
            catalog_cardinality,
            1,
            catalog_cardinality,
            0,
            0,
        )
        .with_max_invalidation_targets(catalog_cardinality),
        evidence_cadence: UiAllocationEvidenceCadence::PerCommittedReceipt,
        collapse_law: UiAllocationStreamCollapseLaw::TerminalOnly,
        partial_settlement_law: UiAllocationPartialSettlementLaw::NotApplicable,
    }
}

pub(crate) fn resolve_stream_families(
    ordered_entries: &[UiAllocationStreamFamily],
    payload_counters: &mut crate::evidence::UiAllocationStreamPolicyPayloadCounters,
) -> UiAllocationStreamCommitDecision {
    let families = canonical_families(ordered_entries, payload_counters);
    let Some(first) = families.first().copied() else {
        return UiAllocationStreamCommitDecision::Denied(
            UiAllocationStreamCompositionDenial::EmptyFrame,
        );
    };
    let contracts = match admitted_pair_contracts(&families, payload_counters) {
        Ok(contracts) => contracts,
        Err(denial) => return UiAllocationStreamCommitDecision::Denied(denial),
    };
    let resolved = resolve_contract_set(first, &contracts, payload_counters);
    let admitted = u16::try_from(ordered_entries.len())
        .expect("a frame carries at most ALLOCATION_FRAME_SOURCE_COUNT ingress");
    if admitted > resolved.budget.ingress_window() {
        return UiAllocationStreamCommitDecision::Denied(
            UiAllocationStreamCompositionDenial::InputBudgetExceeded {
                admitted,
                allowed: resolved.budget.ingress_window(),
            },
        );
    }
    let pair_count = families
        .len()
        .saturating_mul(families.len().saturating_sub(1))
        / 2;
    payload_counters.reserve_vector_capacity(pair_count);
    let mut intermediate = Vec::with_capacity(pair_count);
    for contract in contracts.iter().copied() {
        intermediate.push(UiAllocationIntermediatePolicyVerdict {
            left: contract.left(),
            right: contract.right(),
            outcome: contract.outcome(),
            resolved: contract.resolved(),
        });
    }
    let branches = policy_branches(&families, &intermediate, payload_counters);
    payload_counters.convert_boxed_slice();
    let families = families.into_boxed_slice();
    payload_counters.convert_boxed_slice();
    let intermediate = intermediate.into_boxed_slice();
    let receipt = UiAllocationStreamCompositionReceipt {
        counters: UiAllocationStreamCompositionCounters {
            admitted_family_count: u8::try_from(families.len())
                .expect("a frame composes at most the eight stream families"),
            admitted_input_count: admitted,
            pair_contract_evaluations: payload_counters.pair_contract_evaluations(),
            pair_policy_joins: payload_counters.pair_policy_joins(),
            n_way_policy_joins: payload_counters.n_way_policy_joins(),
            branch_policy_joins: payload_counters.branch_policy_joins(),
        },
        branches,
        families,
        intermediate,
        policy: resolved,
    };
    match resolved.target {
        UiAllocationCommitTarget::PreviewOnly => UiAllocationStreamCommitDecision::Preview(receipt),
        _ => UiAllocationStreamCommitDecision::Commit(receipt),
    }
}

fn policy_branches(
    families: &[UiAllocationStreamFamily],
    verdicts: &[UiAllocationIntermediatePolicyVerdict],
    payload_counters: &mut crate::evidence::UiAllocationStreamPolicyPayloadCounters,
) -> Box<[UiResolvedAllocationPolicyBranch]> {
    payload_counters.reserve_vector_capacity(families.len());
    let mut branches: Vec<Vec<UiAllocationStreamFamily>> = Vec::with_capacity(families.len());
    for family in families.iter().copied() {
        if let Some(composed) = branches.iter_mut().find(|branch| {
            branch.iter().copied().all(|member| {
                verdicts.iter().any(|verdict| {
                    verdict.matches(member, family)
                        && verdict.outcome == UiAllocationFamilyPairOutcome::Compose
                })
            })
        }) {
            composed.push(family);
        } else {
            payload_counters.reserve_vector_capacity(families.len());
            let mut branch = Vec::with_capacity(families.len());
            branch.push(family);
            branches.push(branch);
        }
    }
    payload_counters.reserve_vector_capacity(branches.len());
    let mut resolved_branches = Vec::with_capacity(branches.len());
    for branch in branches {
        let policy = resolve_branch_policy(&branch, verdicts, payload_counters);
        payload_counters.convert_boxed_slice();
        resolved_branches.push(UiResolvedAllocationPolicyBranch {
            families: branch.into_boxed_slice(),
            policy,
        });
    }
    payload_counters.convert_boxed_slice();
    resolved_branches.into_boxed_slice()
}

fn canonical_families(
    entries: &[UiAllocationStreamFamily],
    payload_counters: &mut crate::evidence::UiAllocationStreamPolicyPayloadCounters,
) -> Vec<UiAllocationStreamFamily> {
    payload_counters.reserve_vector_capacity(UiAllocationStreamFamily::ALL.len());
    let mut families = Vec::with_capacity(UiAllocationStreamFamily::ALL.len());
    for family in UiAllocationStreamFamily::ALL {
        if entries.contains(&family) {
            families.push(family);
        }
    }
    families
}

fn admitted_pair_contracts(
    families: &[UiAllocationStreamFamily],
    payload_counters: &mut crate::evidence::UiAllocationStreamPolicyPayloadCounters,
) -> Result<Vec<UiAllocationFamilyPairContract>, UiAllocationStreamCompositionDenial> {
    let pair_count = families.len().saturating_sub(1) * families.len() / 2;
    payload_counters.reserve_vector_capacity(pair_count);
    let mut contracts = Vec::with_capacity(pair_count);
    for left_index in 0..families.len() {
        for right in families.iter().copied().skip(left_index + 1) {
            payload_counters.evaluate_pair_contract();
            let contract = pair_contract(families[left_index], right)?;
            payload_counters.join_pair_policy();
            contracts.push(contract);
        }
    }
    Ok(contracts)
}

fn resolve_contract_set(
    only_family: UiAllocationStreamFamily,
    contracts: &[UiAllocationFamilyPairContract],
    payload_counters: &mut crate::evidence::UiAllocationStreamPolicyPayloadCounters,
) -> UiResolvedAllocationStreamPolicy {
    let mut policies = contracts.iter().map(|contract| contract.resolved());
    let Some(mut resolved) = policies.next() else {
        return resolved_family_policy(only_family);
    };
    for policy in policies {
        payload_counters.join_n_way_policy();
        resolved = join_contract_policies(resolved, policy);
    }
    resolved
}

fn resolve_branch_policy(
    branch: &[UiAllocationStreamFamily],
    verdicts: &[UiAllocationIntermediatePolicyVerdict],
    payload_counters: &mut crate::evidence::UiAllocationStreamPolicyPayloadCounters,
) -> UiResolvedAllocationStreamPolicy {
    let mut policies = verdicts
        .iter()
        .filter(|verdict| {
            branch.contains(&verdict.left)
                && branch.contains(&verdict.right)
                && verdict.outcome == UiAllocationFamilyPairOutcome::Compose
        })
        .map(|verdict| verdict.resolved);
    let Some(mut resolved) = policies.next() else {
        return resolved_family_policy(branch[0]);
    };
    for policy in policies {
        payload_counters.join_branch_policy();
        resolved = join_contract_policies(resolved, policy);
    }
    resolved
}

impl UiAllocationIntermediatePolicyVerdict {
    fn matches(&self, left: UiAllocationStreamFamily, right: UiAllocationStreamFamily) -> bool {
        (self.left == left && self.right == right) || (self.left == right && self.right == left)
    }
}

#[cfg(test)]
mod tests;
