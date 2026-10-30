//! A genuine copied branch shares entity occurrences but is not the model's owned head.
use super::super::super::{
    access,
    application::{
        candidate::{PreparationFailure, PreparationRefusal, Staging},
        model::Model,
        storage::{ProposalReference, Storage},
    },
    authentication::Caller,
    host::Host,
    recording, storage_query,
};
use worth_query_host::facade::{
    application_entry::*, product::*, runtime::ExecutionAllocationPolicy,
};

fn describe(
    host: &Host,
    caller: &Caller<'_>,
    branch: WorthQueryProductBranch,
) -> WorthQueryExpectedProductBasisDescription {
    host.with_application_advancement(caller.scope, |phase| {
        host.on_branch(branch)
            .select(&phase)
            .unwrap()
            .describe_basis()
    })
    .unwrap()
}
pub(super) fn refuse_copied_source(
    host: &Host,
    model: &Model<'_>,
    storage: &Storage<'_>,
    caller: &Caller<'_>,
    original: &ProposalReference,
) {
    let copied = Storage::create(model, caller).unwrap();
    assert_ne!(copied.branch(), model.branch());
    let mut data = storage.load(caller, original).unwrap();
    data.expected_basis = serde_json::to_string(&describe(host, caller, copied.branch())).unwrap();
    let outcome = host
        .request(&caller.principal, caller.scope)
        .on_branch(storage.branch())
        .advanced()
        .mutate(recording::RecordProposal {
            model: model.identity.label.clone(),
            key: "copied-source".to_owned(),
            data: data.clone(),
        })
        .without_source()
        .idempotency(&705)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let rows = host
        .request(&caller.principal, caller.scope)
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
    let mut matching = rows.rows().iter().filter(|row| row.key == "copied-source");
    let row = matching.next().expect("ordinary stored record");
    assert!(matching.next().is_none());
    assert_eq!(row.data, data);
    let reference = ProposalReference {
        key: row.key.clone(),
        occurrence: row.occurrence.clone(),
        model: row.data.model.clone(),
        revision: row.data.revision,
    };
    let before = model.describe(caller).unwrap();
    let storage_before = describe(host, caller, storage.branch());
    let Err(PreparationFailure::Refused {
        cause: PreparationRefusal::ModelHeadMismatch,
        handoff: None,
    }) = Staging::prepare(model, storage, caller, &reference, 706)
    else {
        panic!(
            "shared genuine model/target occurrences cannot substitute for the actual model head"
        )
    };
    assert_eq!(model.describe(caller).unwrap(), before);
    assert_eq!(model.read(caller).unwrap().days, 3);
    assert_eq!(describe(host, caller, storage.branch()), storage_before);
    assert_eq!(storage.load(caller, &reference).unwrap(), data);
    copied.close(caller).unwrap();
}
