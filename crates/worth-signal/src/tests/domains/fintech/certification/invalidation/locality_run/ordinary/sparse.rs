use std::sync::OnceLock;

use super::super::super::locality_completion::certify_contract;
use crate::tests::domains::fintech::world::SparseFanoutAxis;

use super::*;

pub(super) fn contract_evidence(
    total_outputs: u32,
    axis: SparseFanoutAxis,
) -> &'static [FinancialLocalityCaseEvidence] {
    static CONTRACTS: [OnceLock<Vec<FinancialLocalityCaseEvidence>>; 9] =
        [const { OnceLock::new() }; 9];
    let contracts = ordinary_locality_cases()
        .into_iter()
        .filter(|case| case.scenario() == FinancialLocalityScenario::SparseBookFanout)
        .collect::<Vec<_>>();
    assert_eq!(contracts.len(), CONTRACTS.len());
    let scale = LocalityScaleTuple::SparseBookFanout {
        total_outputs,
        axis,
    };
    let index = contracts
        .iter()
        .position(|case| case.scale == scale)
        .unwrap();
    CONTRACTS[index].get_or_init(|| certify_contract(41, contracts[index]).unwrap())
}

fn assert_contract(total_outputs: u32, axis: SparseFanoutAxis) {
    let cases = contract_evidence(total_outputs, axis);
    assert_ordinary_cases(FinancialLocalityScenario::SparseBookFanout, cases);
    for case in cases {
        assert_eq!(
            case.scale(),
            LocalityScaleTuple::SparseBookFanout {
                total_outputs,
                axis
            }
        );
        println!(
            "SPARSE_CONTRACT_EVIDENCE axis={axis:?} outputs={total_outputs} elapsed_seconds={} canonical_work={} necessary_evaluations={} counters={:?}",
            case.measurement().elapsed().as_secs_f64(),
            case.canonical_work_items(),
            case.necessary_evaluation_count(),
            case.counters().values(),
        );
    }
}

#[test]
fn ordinary_index_disjoint_64_completes() {
    assert_contract(64, SparseFanoutAxis::IndexDisjoint);
}

#[test]
fn ordinary_index_disjoint_512_completes() {
    assert_contract(512, SparseFanoutAxis::IndexDisjoint);
}

#[test]
fn ordinary_index_disjoint_4096_completes() {
    assert_contract(4_096, SparseFanoutAxis::IndexDisjoint);
}

#[test]
fn ordinary_queried_rejecting_64_completes() {
    assert_contract(64, SparseFanoutAxis::QueriedRejecting);
}

#[test]
fn ordinary_queried_rejecting_512_completes() {
    assert_contract(512, SparseFanoutAxis::QueriedRejecting);
}

#[test]
fn ordinary_queried_rejecting_4096_completes() {
    assert_contract(4_096, SparseFanoutAxis::QueriedRejecting);
}

#[test]
fn ordinary_rejected_descendants_64_completes() {
    assert_contract(64, SparseFanoutAxis::RejectedDescendants);
}

#[test]
fn ordinary_rejected_descendants_512_completes() {
    assert_contract(512, SparseFanoutAxis::RejectedDescendants);
}

#[test]
fn ordinary_rejected_descendants_4096_completes() {
    assert_contract(4_096, SparseFanoutAxis::RejectedDescendants);
}
