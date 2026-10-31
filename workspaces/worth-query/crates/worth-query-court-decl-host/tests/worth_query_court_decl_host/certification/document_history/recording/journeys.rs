use super::*;

#[test]
fn ordinary_recording_keeps_candidate_private_then_selects_one_genuine_step() {
    let host = publish_on_first_program();
    let owned = head(&host);
    let original = owned.branch();
    let document = attach(&host, owned, DOCUMENT_IDENTITY);
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let mut edit = begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_0400);
    for expected in [
        Posture::CreatingCandidate,
        Posture::Running,
        Posture::AwaitingSelection,
    ] {
        let Advance::Progress(next) = edit.advance(&request) else {
            panic!("ordinary progression")
        };
        edit = next;
        assert_eq!(edit.posture(), expected);
        assert_eq!(edit.read(&request).unwrap().rows()[0].retention_days, 7);
    }
    let candidate_commit = {
        let WorthQuerySourceCompletion::Fresh { receipt, .. } =
            edit.completed_source().unwrap().evidence()
        else {
            panic!("fresh actual completion")
        };
        assert_eq!(read_retention(host.runtime(), receipt.product_branch()), 3);
        host.with_host_advancement(|phase| {
            let selected = host
                .runtime()
                .on_branch(receipt.product_branch())
                .select(&phase)
                .unwrap();
            assert_eq!(
                selected.product().selected_commit(),
                receipt.committed_product_publication().composite_commit()
            );
            selected.product().selected_commit().clone()
        })
        .unwrap()
    };
    assert_eq!(read_retention(host.runtime(), original), 7);
    assert_eq!(value(&host, &untouched), 7);
    let Advance::Ready(edit) = edit.advance(&request) else {
        panic!("actual adoption returns selected ownership")
    };
    assert_eq!(edit.posture(), Posture::Selected);
    assert_eq!(edit.read(&request).unwrap().rows()[0].retention_days, 3);
    let (document, step) = finish(&host, edit, 3);
    assert_eq!(step.parent(), Some(root.id()));
    assert_eq!(step.label(), Some("Set retention"));
    assert_eq!(step.endpoint().selected_commit(), &candidate_commit);
    assert_eq!(document.current_step(&request).unwrap().id(), step.id());
    assert_eq!(value(&host, &untouched), 7);
    close(&host, document, 2);
    close(&host, untouched, 1);
}

#[test]
fn retry_finds_the_original_occurrence_after_later_edits_without_resetting_selection() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let (document, first) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_0401),
        ),
        3,
    );
    let (document, second) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 7, 0x9182_0402),
        ),
        7,
    );
    let (document, third) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_0403),
        ),
        3,
    );
    assert_ne!(first.id(), third.id());
    assert_ne!(
        first.endpoint().selected_commit(),
        third.endpoint().selected_commit()
    );
    assert_eq!(second.parent(), Some(first.id()));
    assert_eq!(third.parent(), Some(second.id()));
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let retry = begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_0401);
    let Advance::Ready(retry) = retry.advance(&request) else {
        panic!("known attempt needs no fork or business advancement")
    };
    assert_eq!(retry.posture(), Posture::AlreadyRecorded);
    assert_eq!(retry.selected_step().unwrap().id(), first.id());
    let finished = retry.finish(&request).unwrap();
    let Outcome::AlreadyRecorded { step, .. } = finished.outcome else {
        panic!("original outcome")
    };
    assert_eq!(step.id(), first.id());
    assert_eq!(
        finished.document.current_step(&request).unwrap().id(),
        third.id()
    );
    let selected = finished.document.read(&request).unwrap();
    assert_eq!(selected.rows()[0].retention_days, 3);
    let third_branch = host
        .with_host_advancement(|phase| {
            host.runtime()
                .on_branch(third.endpoint().product_branch())
                .select(&phase)
                .unwrap()
                .product()
                .relational_basis_descriptor()
                .branch_id()
                .0
                .clone()
        })
        .unwrap();
    assert_eq!(selected.receipt().inspect().basis().branch(), third_branch);
    drop(selected);
    close(&host, finished.document, 4);
}
