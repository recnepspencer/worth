//! Every stage of a recorded envelope is scoped inside the document's model.
use super::*;
use worth_query_host::facade::product::WorthQueryProductBranch;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::identity::EntityId;
use WorthQueryDocumentHistoryDenial::UnownedOperationScope;
use WorthQueryHistoryEditEnvelopeDenial as Denial;
use WorthQueryHistoryEditEnvelopeOutcome as Outcome;

type Execution<'a> = Result<
    WorthQueryHistoryEditEnvelopeOutcome<'a, Schema, History>,
    WorthQueryHistoryEditEnvelopeFailure<'a, Schema, History>,
>;

/// One authored stage. Model `a` is the document's model; `b` is the other.
enum Stage {
    EditModel(u64),
    EditMember(&'static str, u64),
    MoveToOtherModel(&'static str),
    AddMember(&'static str, u64),
}

fn branch(host: &world::Host, document: &world::Document<'_>) -> WorthQueryProductBranch {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    document
        .selection(&host.request(&principal, &scope))
        .unwrap()
        .physical_endpoint()
        .product_branch()
}

/// Author the stages on the document and execute them as one recorded step.
fn record<'a>(
    host: &'a world::Host,
    document: world::Document<'a>,
    key: &str,
    stages: &[Stage],
) -> Execution<'a> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host
        .request(&principal, &scope)
        .on_branch(branch(host, &document));
    let mut envelope = document.edit_envelope(host, &request, key.to_owned())?;
    for stage in stages {
        match *stage {
            Stage::EditModel(value) => {
                envelope
                    .push_stage::<ApplicationNoOutputGraph, _>(edit::model::Intent(edit::Edit {
                        key: "a".into(),
                        value,
                    }))
                    .unwrap();
            }
            Stage::EditMember(key, value) => {
                envelope
                    .push_stage::<ApplicationNoOutputGraph, _>(edit::member::Intent(edit::Edit {
                        key: key.into(),
                        value,
                    }))
                    .unwrap();
            }
            Stage::MoveToOtherModel(member) => {
                envelope
                    .push_stage::<ApplicationNoOutputGraph, _>(ownership_change::Change {
                        from: "a".into(),
                        to: "b".into(),
                        member: member.into(),
                        preserve_previous: false,
                    })
                    .unwrap();
            }
            Stage::AddMember(key, value) => {
                envelope
                    .push_stage::<ApplicationNoOutputGraph, _>(birth::Intent {
                        key: key.into(),
                        value,
                    })
                    .unwrap();
            }
        }
    }
    envelope.execute_recorded(
        WorthQueryHistoryStepMetadata::from_bytes(Vec::new()),
        ExecutionAllocationPolicy::SystemAllocation,
    )
}

fn recorded(execution: Execution<'_>) -> (world::Document<'_>, WorthQueryHistoryStep) {
    match execution {
        Ok(Outcome::Recorded { document, step, .. }) => (document, step),
        Ok(other) => panic!("the envelope must record: {other:?}"),
        Err(failure) => panic!("the envelope must record: {failure:?}"),
    }
}

/// The refused stage and the scope and model its refusal names.
fn refused_as_unowned(execution: Execution<'_>) -> (world::Document<'_>, usize, EntityId, EntityId) {
    let (document, denial) = match execution {
        Err(failure) => failure.into_parts(),
        Ok(other) => panic!("the envelope must refuse: {other:?}"),
    };
    let Denial::Stage {
        ordinal,
        denial: UnownedOperationScope { subject, model },
    } = denial
    else {
        panic!("{denial:?}")
    };
    (document, ordinal, subject, model)
}

fn occurrence(entity: EntityId) -> (u32, u64, u32) {
    (
        entity.partition_id.as_u32(),
        entity.local_slot_value(),
        entity.generation_value(),
    )
}

fn model_value(host: &world::Host, branch: WorthQueryProductBranch, key: &str) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.request(&principal, &scope)
        .on_branch(branch)
        .query(query::model::Read(key.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .value
}

fn current(host: &world::Host, document: &world::Document<'_>) -> WorthQueryHistoryStep {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    document
        .current_step(&host.request(&principal, &scope))
        .unwrap()
}

/// Closing disposes every head the document still holds, including the
/// candidate of a refused envelope.
fn close(host: &world::Host, document: world::Document<'_>) {
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn a_foreign_model_stage_refuses_before_effects() {
    let host = world::install();
    let document = world::attach(&host);
    let head = current(&host, &document);
    let own = branch(&host, &document);
    let stages = [Stage::EditModel(2), Stage::EditMember("foreign", 34)];
    let (document, ordinal, subject, model) =
        refused_as_unowned(record(&host, document, "foreign-stage", &stages));
    assert_eq!(ordinal, 1);
    assert_eq!(
        occurrence(subject),
        world::member_identity(&host, own, "foreign")
    );
    assert_eq!(occurrence(model), world::model_identity(&host, own, "a"));
    assert_eq!(current(&host, &document), head);
    assert_eq!(branch(&host, &document), own);
    assert_eq!(model_value(&host, own, "a"), 1, "stage 0 was not kept");
    assert_eq!(world::member_value(&host, own, "foreign"), 33);
    assert_eq!(model_value(&host, own, "b"), 1);
    close(&host, document);
}

#[test]
fn an_unowned_existing_scope_refuses_at_admission() {
    let host = world::install();
    let document = world::attach(&host);
    let head = current(&host, &document);
    let own = branch(&host, &document);
    let stages = [Stage::EditModel(2), Stage::EditMember("loose", 56)];
    let (document, ordinal, subject, _) =
        refused_as_unowned(record(&host, document, "unowned-stage", &stages));
    assert_eq!(ordinal, 1);
    assert_eq!(
        occurrence(subject),
        world::member_identity(&host, own, "loose")
    );
    assert_eq!(current(&host, &document), head);
    assert_eq!(model_value(&host, own, "a"), 1);
    assert_eq!(world::member_value(&host, own, "loose"), 55);
    close(&host, document);
}

/// The head owns `direct`, so its stage is admitted; the earlier stage of the
/// same step hands it to the other model, so the committed endpoint refuses.
#[test]
fn a_stage_whose_scope_leaves_the_model_within_the_step_refuses_at_the_endpoint() {
    let host = world::install();
    let document = world::attach(&host);
    let head = current(&host, &document);
    let own = branch(&host, &document);
    let direct = world::member_identity(&host, own, "direct");
    let stages = [
        Stage::MoveToOtherModel("direct"),
        Stage::EditMember("direct", 12),
    ];
    let (document, ordinal, subject, model) =
        refused_as_unowned(record(&host, document, "moved-away", &stages));
    assert_eq!(ordinal, 1);
    assert_eq!(occurrence(subject), direct);
    assert_eq!(occurrence(model), world::model_identity(&host, own, "a"));
    assert_eq!(current(&host, &document), head);
    assert_eq!(branch(&host, &document), own);
    assert_eq!(world::member_value(&host, own, "direct"), 11);
    // The refused commit stayed private: the same key now records on the same head.
    let valid = [Stage::EditModel(2), Stage::EditMember("direct", 12)];
    let (document, step) = recorded(record(&host, document, "moved-away", &valid));
    assert_eq!(step.parent(), Some(head.id()));
    let after = branch(&host, &document);
    assert_eq!(model_value(&host, after, "a"), 2);
    assert_eq!(world::member_value(&host, after, "direct"), 12);
    close(&host, document);
}

#[test]
fn a_model_scoped_birth_stage_and_an_owned_member_stage_record_one_step() {
    let host = world::install();
    let document = world::attach(&host);
    let head = current(&host, &document);
    let stages = [Stage::AddMember("born", 7), Stage::EditMember("direct", 12)];
    let (document, birth) = recorded(record(&host, document, "birth", &stages));
    assert_eq!(birth.parent(), Some(head.id()));
    assert_eq!(birth.label(), Some("Add member"));
    assert_eq!(current(&host, &document), birth);
    let after = branch(&host, &document);
    assert_eq!(world::member_value(&host, after, "born"), 7);
    assert_eq!(world::member_value(&host, after, "direct"), 12);
    // The model owns the newborn at the new head, so a member stage on it is admitted.
    let next = [Stage::EditModel(2), Stage::EditMember("born", 8)];
    let (document, step) = recorded(record(&host, document, "edit-newborn", &next));
    assert_eq!(step.parent(), Some(birth.id()));
    assert_eq!(
        world::member_value(&host, branch(&host, &document), "born"),
        8
    );
    close(&host, document);
}
