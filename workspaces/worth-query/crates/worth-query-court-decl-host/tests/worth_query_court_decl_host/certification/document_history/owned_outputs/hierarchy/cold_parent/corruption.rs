//! Wrong cold Query parent lineage cannot impersonate the authentic Native publication.
use super::*;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let (checkpoint, original, _) = fixture::checkpoint();
    let checkpoint = checkpoint.rewrite_authenticated_checkpoint_body_for_test(|body| {
        fn skip_text(body: &[u8], at: &mut usize) {
            let count = u64::from_be_bytes(body[*at..*at + 8].try_into().unwrap()) as usize;
            *at += 8 + count;
        }
        let native = u64::from_be_bytes(body[14..22].try_into().unwrap()) as usize;
        let mut at = 30 + native;
        skip_text(body, &mut at);
        at += 1 + 48 + 32 + 16 + 32 + 1 + 32 + 32;
        assert!(u64::from_be_bytes(body[at..at + 8].try_into().unwrap()) > 0);
        at += 8;
        skip_text(body, &mut at);
        at += 1;
        skip_text(body, &mut at);
        assert_eq!(
            u32::from_be_bytes(body[at + 12..at + 16].try_into().unwrap()),
            original.0.native_id.generation_value()
        );
        body[at + 12..at + 16].copy_from_slice(&u32::MAX.to_be_bytes());
    });
    let host = world::restore_program_checkpoint::<original::Program>(checkpoint, worker, false);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let (document, before) = fixture::record(&host, &request, world::attach(&host));
    let current = document.selection(&request).unwrap();
    let at_c = host
        .request(&principal, &scope)
        .on_branch(current.physical_endpoint().product_branch());
    let model = at_c
        .query(query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<original::Program, original::Tree>(
                    &phase,
                    &request,
                    &host,
                    first::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let refusal = super::super::super::courts::checkout(
        &document,
        &request,
        before,
        current.clone(),
        0x0d12_1441,
    )
    .expect_err("wrong parent lineage must refuse atomically");
    assert_eq!(document.selection(&request).unwrap(), current);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    let (
        _,
        WorthQueryHistoryNavigationAdvanceDenial::Outputs(
            WorthQueryHistoryOutputDenial::Navigation(
                WorthQueryHistoryNavigationDenial::OutputPublicationCorrespondence(subject),
            ),
        ),
    ) = &refusal
    else {
        panic!("{refusal:?}");
    };
    assert_eq!(subject.source(), model.native_id);
    assert_eq!(subject.family(), "first");
    assert_eq!(subject.producer(), "owned-outputs.first.initial");
    assert_eq!(
        at_c.query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows(),
        &[model]
    );
    assert_eq!(
        at_c.query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows(),
        &[original.0]
    );
    assert_eq!(
        at_c.query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows(),
        &[original.1]
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn corrupt_non_leaf_parent_lineage_refuses_without_effect() {
    exercise(false);
}
#[test]
fn worker_only_corrupt_non_leaf_parent_lineage_refuses_without_effect() {
    exercise(true);
}
