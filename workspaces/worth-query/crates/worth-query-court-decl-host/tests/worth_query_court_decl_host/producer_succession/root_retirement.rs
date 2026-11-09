//! A lone installed root preserves its own validated descendant retirement proof.
use super::{settlement::*, *};
use producers::parent::{
    self,
    preserve::{Outputs, RetiredRealizations},
};

#[test]
fn lone_root_retirement_settles_once_and_reuses_its_closure_evidence() {
    let selections = Selections::default();
    let app = world::install(2, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    close(settle(&app, &request), &request);
    let retired = ["root.1", "root.1.0"].map(|key| {
        request
            .query(realization_read::ReadRealization(key.into()))
            .execute()
            .unwrap()
            .rows()[0]
            .native_id
    });
    resize(&app, &request, 1, 720);
    selections.lock().unwrap().clear();
    let mut first_commit = None;
    for expected_contacts in [1, 0] {
        // This handle demands only the root, without a coupled producer cohort
        // or the program's dependent-output traversal.
        let mut demand = request
            .demand(parent::Demand("root".into()))
            .start_in_program::<Program, program::Tree>(&app)
            .unwrap();
        let settled = loop {
            match demand.settle(&request).unwrap_or_else(|denial| {
                panic!(
                    "expected_contacts={expected_contacts}; calls={:?}; {denial:?}",
                    selections.lock().unwrap()
                )
            }) {
                WorthQueryApplicationOutputDemandProgress::Settled(settled) => break settled,
                WorthQueryApplicationOutputDemandProgress::Progressed
                | WorthQueryApplicationOutputDemandProgress::Pending => continue,
            }
        };
        let closure = settled
            .closure_evidence_of::<Outputs>()
            .unwrap()
            .family_entries::<RetiredRealizations>()
            .unwrap()
            .into_iter()
            .map(|entry| (entry.role.to_owned(), entry.entity_id))
            .collect::<Vec<_>>();
        assert_eq!(
            settled.producer_contacts_in_this_demand(),
            expected_contacts,
            "one shrink publication; unchanged repeat reuses it; closure: {closure:?}"
        );
        assert_eq!(
            closure
                .iter()
                .map(|(role, id)| (role.as_str(), *id))
                .collect::<Vec<_>>(),
            vec![
                ("retired-realization.root.1", retired[0]),
                ("retired-realization.root.1.0", retired[1])
            ]
        );
        assert!(settled
            .outputs_of::<Outputs>()
            .unwrap()
            .family_entries::<RetiredRealizations>()
            .unwrap()
            .is_empty());
        let commit = settled.observation().selected_commit().clone();
        if let Some(first) = first_commit {
            assert_eq!(commit, first);
        }
        first_commit = Some(commit);
        demand.close();
    }
    let calls = selections.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].key, "root");
    assert!(calls[0].preserve);
    assert_absent(&request, &["root.1", "root.1.0"]);
}
