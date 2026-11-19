//! Contact expectations come from the actual in-edit producer's Native evidence.
use super::*;

pub(in super::super) fn paired_edit(
    application: &host::OutputHost,
    receipt: &WorthQueryApplicationCommitReceipt,
    value: u64,
    commits: usize,
    contacts: u64,
) -> Option<usize> {
    assert_eq!(
        commits, 1,
        "one actual Native source commit across all branches"
    );
    let completed = application
        .runtime()
        .runtime_for_test()
        .primary_provider_for_test()
        .observe_completed_application_for_test(receipt.commit_reference())
        .unwrap();
    let mut members = completed.coupled_members_for_test();
    let Some(producer) = members.next() else {
        assert_eq!(
            contacts, 0,
            "a verified no-op source reuses its actual producer"
        );
        return None;
    };
    assert!(
        members.next().is_none(),
        "this installed source has exactly one real producer"
    );
    assert_eq!(
        producer.commit_reference_for_test(),
        receipt.commit_reference()
    );
    assert_eq!(
        contacts, 1,
        "the producer really executed inside this source edit"
    );
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let output = application
        .runtime()
        .request(&principal, &scope)
        .query(super::super::output::query::Read {
            key: "numeric".into(),
        })
        .execute_governed(
            super::super::super::causal_inspection::capability::Inspect::reference(),
            super::super::super::causal_inspection::capability::InspectCausalEvidence::reference(),
            super::super::super::causal_inspection::capability::Input::at("numeric"),
        )
        .unwrap();
    assert_eq!(output.rows().len(), 1);
    assert_eq!(
        output.rows()[0].total,
        Some(17 + 7 * (value % 4)),
        "output exists before delivery starts"
    );
    Some(
        producer
            .commit_evidence()
            .source_facts_superseded_by_own_effect()
            .count(),
    )
}

pub(super) fn paired_delivery(
    source: &WorthQueryApplicationCommitReceipt,
    producer: Option<&WorthQueryApplicationCommitReceipt>,
    own_effect: Option<usize>,
    commits: usize,
    contacts: u64,
    handle_contacts: usize,
) {
    let expected = usize::from(own_effect.is_some_and(|facts| facts != 0));
    assert_eq!(
        contacts, expected as u64,
        "only actual own-effect supersession requires reobservation"
    );
    assert_eq!(handle_contacts, expected);
    assert_eq!(
        commits, expected,
        "master's real successor publication is retained"
    );
    if let Some(facts) = own_effect {
        let producer = producer.expect("a performed producer retains its real receipt");
        if facts == 0 {
            assert_eq!(producer.commit_reference(), source.commit_reference());
        } else {
            assert_ne!(producer.commit_reference(), source.commit_reference());
            assert_eq!(
                producer.commit_reference().parents.as_slice(),
                &[source.commit_reference().commit_id]
            );
        }
    }
}
