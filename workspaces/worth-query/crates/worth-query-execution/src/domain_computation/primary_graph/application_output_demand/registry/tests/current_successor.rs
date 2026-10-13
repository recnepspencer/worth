//! The queued cause and its answering successor have distinct physical rows.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::CurrentProof;

#[test]
fn registry_current_successor_pin_acknowledges_only_its_native_current_proof() {
    for foreign in [false, true] {
        let registry = WorthQueryOutputDemandRegistry::default();
        let mut old = key_with_identity("answering-producer", 1, 1, 10);
        old.applicability = crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerApplicability::new(
            "registry-fixture", crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerLifecyclePosture::Initial);
        let newest = key_with_identity("answering-producer", 2, 1, 11);
        let other = key("foreign-answer", 2, 2);
        let (old_member, answer_member, other_member) = {
            let mut state = registry.state.lock().unwrap();
            state.records.insert(
                old.clone(),
                record(
                    occurrence(),
                    DemandState::Failed(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "",
                    )),
                    1,
                ),
            );
            state.records.insert(
                newest.clone(),
                record(occurrence(), closed_retirement::ready(), 1),
            );
            state.records.insert(
                other.clone(),
                record(occurrence(), closed_retirement::ready(), 1),
            );
            for key in [&old, &newest, &other] {
                retain_owned_key(&mut state, key);
                let member = state.prepare_required_member(key).unwrap();
                state.install_required_member(member);
            }
            (
                Arc::clone(state.records[&old].work_membership.as_ref().unwrap()),
                Arc::clone(state.records[&newest].work_membership.as_ref().unwrap()),
                Arc::clone(state.records[&other].work_membership.as_ref().unwrap()),
            )
        };
        let mut work = old_member.fixture_selected_work();
        registry
            .pin_current_required_successor(&mut work, &mut record_admission())
            .unwrap();
        // The cause stays on the predecessor even though its answer is newer.
        assert_eq!(work.key(), &old);
        CurrentProof::with_current_fixture_basis(
            if foreign {
                &other_member
            } else {
                &answer_member
            },
            |proof| {
                let result = work.acknowledge_current_admitted(proof, &mut record_admission());
                if foreign {
                    assert!(
                        matches!(result, Err(ref denial) if denial.kind() == WorthQueryOutputDemandDenialKind::ForeignDemand)
                    );
                    assert!(old_member.fixture_is_unresolved());
                } else {
                    assert!(result.is_ok());
                    assert!(!old_member.fixture_is_unresolved());
                }
            },
        );
    }
}
