//! A genuinely stored unsupported format refuses before registration or candidate creation.
use super::*;
use crate::proposal_acceptance::{
    application::candidate::PreparationRefusal, recording, schema::data::ProposalReference,
    storage_query,
};

#[test]
fn unsupported_stored_format_does_not_claim_a_key_or_replace_the_supported_proposal() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let reviewer =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &scope).unwrap();
    let mut model = Model::create(&host, &author, "model-1", 1601, 7).unwrap();
    let other = Model::create(&host, &author, "model-2", 1602, 5).unwrap();
    let storage = Storage::create(&model, &author).unwrap();
    let supported = storage
        .record(
            &model,
            &author,
            "proposal-1",
            1603,
            vec![AuthoredIntent::Retention(edit(&model, 3))],
        )
        .unwrap();
    let supported = storage.load_frozen(&reviewer, &supported).unwrap();
    let mut data = supported.proposal().clone();
    data.format_version = 2;
    assert_eq!(data.revision, 1);
    let outcome = host
        .request(&author.principal, author.scope)
        .on_branch(storage.branch())
        .advanced()
        .mutate(recording::RecordProposal {
            model: model.identity.label.clone(),
            key: "proposal-unsupported".to_owned(),
            data: data.clone(),
        })
        .without_source()
        .idempotency(&1604)
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    drop(outcome);
    let reference = {
        let rows = host
            .request(&reviewer.principal, reviewer.scope)
            .on_branch(storage.branch())
            .query(storage_query::ReadProposals {
                model: model.identity.label.clone(),
            })
            .execute_governed(
                access::ReadDomain::reference(),
                access::InspectDomain::reference(),
                access::ReadAccess {
                    model: model.identity.label.clone(),
                },
            )
            .unwrap();
        let mut matching = rows
            .rows()
            .iter()
            .filter(|row| row.key == "proposal-unsupported");
        let row = matching
            .next()
            .expect("the ordinary operation issued the stored row");
        assert!(matching.next().is_none());
        assert_eq!(row.data, data);
        ProposalReference {
            key: row.key.clone(),
            occurrence: row.occurrence.clone(),
            model: row.data.model.clone(),
            revision: row.data.revision,
        }
    };
    let unsupported = storage.load_frozen(&reviewer, &reference).unwrap();
    assert_eq!(unsupported.proposal(), &data);
    assert_eq!(unsupported.proposal().format_version, 2);
    assert_eq!(unsupported.proposal().revision, 1);
    let before = model.describe(&reviewer).unwrap();
    let storage_before = describe(&host, &reviewer, storage.branch());
    assert!(matches!(
        model.accept(&storage, &reviewer, &unsupported, 1605),
        Err(Failure::Eligibility(
            PreparationRefusal::UnsupportedVersion(2)
        ))
    ));
    assert!(model
        .inspect_attempt(&storage, &reviewer, &unsupported, 1605)
        .unwrap()
        .is_none());
    let resolution = host
        .request(&reviewer.principal, reviewer.scope)
        .on_branch(storage.branch())
        .advanced()
        .mutate(unsupported.input(1605))
        .without_source()
        .idempotency(&1605)
        .resolve_idempotency_in_program(&host)
        .unwrap()
        .into_resolution();
    assert!(matches!(
        resolution,
        WorthQueryApplicationIdempotencyResolution::Unseen
    ));
    assert_eq!(model.describe(&reviewer).unwrap(), before);
    assert_eq!(describe(&host, &reviewer, storage.branch()), storage_before);
    assert_eq!(model.read(&reviewer).unwrap().days, 7);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    assert_eq!(supported.proposal().format_version, 1);
    assert_eq!(supported.proposal().revision, 1);
    reach_adoption(&mut model, &storage, &reviewer, &supported, 1606);
    assert_eq!(model.read(&reviewer).unwrap().days, 3);
    assert_eq!(other.read(&reviewer).unwrap().days, 5);
    drop(finish(&mut model, &storage, &reviewer, &supported, 1606));
    storage.close(&author).unwrap();
    model.close(&author).unwrap();
    other.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
