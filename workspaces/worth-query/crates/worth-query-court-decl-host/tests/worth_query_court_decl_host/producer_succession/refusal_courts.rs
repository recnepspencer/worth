//! Deliberately incomplete and foreign candidates still enter real Query producers.
use super::{settlement::*, *};
fn refusal(fault: Fault) -> worth_relational::facade::publication::PublicationError {
    let app = world::install_fault(fault);
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    close(settle(&app, &request), &request);
    let source_id =
        |key: &str| request.query(Read(key.into())).execute().unwrap().rows()[0].native_id;
    let output_id = |key: &str| {
        request
            .query(realization_read::ReadRealization(key.into()))
            .execute()
            .unwrap()
            .rows()[0]
            .native_id
    };
    use worth_relational::facade::transactions::RecordRef;
    let subjects = match fault {
        Fault::Missing => vec![
            RecordRef::Entity(output_id("root.1")),
            RecordRef::Entity(source_id("root.1.0")),
            RecordRef::Entity(source_id("root.1")),
        ],
        Fault::MissingLeaf => vec![RecordRef::Entity(output_id("root.1.0"))],
        Fault::Partial => vec![RecordRef::Entity(source_id("root.1.0"))],
        Fault::Foreign => vec![RecordRef::Entity(output_id("root.0"))],
        Fault::Extra | Fault::HiddenDeletion => vec![RecordRef::Entity(output_id("unrelated"))],
        Fault::Repeated => vec![RecordRef::Entity(source_id("root.1"))],
        Fault::None => unreachable!("refusal court selects a malformed candidate"),
    };
    resize(&app, &request, 1, 1);
    let mut operation = request
        .demand(producers::parent::Demand("root".into()))
        .retain_program_outputs::<Program, program::Tree>(&app)
        .unwrap_or_else(|e| panic!("{:?}", e.denial()));
    let mut failure = None;
    let require_hidden_subject_live = || {
        if fault == Fault::HiddenDeletion {
            assert!(
                request
                    .query(realization_read::ReadRealization("unrelated".into()))
                    .execute()
                    .is_ok(),
                "hidden ordinary entity was deleted before refusal"
            );
        }
    };
    for _ in 0..128 {
        match operation.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                require_hidden_subject_live();
                failure = Some(original_error(&denial));
                break;
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                require_hidden_subject_live();
                let error = original_error(terminal.denial());
                terminal
                    .close(&request)
                    .unwrap_or_else(|e| panic!("{:?}", e.denial()));
                failure = Some(error);
                break;
            }
            WorthQueryProgramOutputOperationAdvance::Settled(_) => {
                panic!("{fault:?} illegally committed")
            }
        }
    }
    let error = failure.expect("malformed candidate must terminate with a refusal");
    if fault == Fault::Missing {
        assert_eq!(error.context.affected_records.len(), 2);
        assert!(subjects[..2].contains(&error.context.affected_records[0]));
        assert_eq!(error.context.affected_records[1], subjects[2]);
    } else {
        assert_eq!(
            error.context.affected_records, subjects,
            "original native subjects"
        );
    }
    assert_live(
        &request,
        &["root", "root.0", "root.0.0", "root.1", "root.1.0"],
    );
    if fault == Fault::HiddenDeletion {
        assert_eq!(RecordRef::Entity(output_id("unrelated")), subjects[0]);
    }
    error
}
#[test]
fn deleted_member_without_its_owned_closure_refuses_with_the_missing_publication_subject() {
    let error = refusal(Fault::Missing);
    assert!(matches!(error.authored_retirement_refusal(), Some(worth_relational::facade::publication::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { .. })));
}
#[test]
fn partial_owned_publication_refuses_with_the_missing_member_subject() {
    let error = refusal(Fault::Partial);
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::MissingMember)
    );
}
#[test]
fn a_foreign_owned_publication_is_not_adopted_by_the_shrinking_generator() {
    let error = refusal(Fault::Foreign);
    assert_eq!(error.authored_retirement_refusal(), Some(worth_relational::facade::publication::AuthoredRetirementRefusal::DifferentSourcePublication));
}
#[test]
fn repeated_retirement_names_the_repeated_subject() {
    let error = refusal(Fault::Repeated);
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::RepeatedTarget)
    );
}

fn original_error(
    denial: &WorthQueryProgramOutputAdvanceDenial,
) -> worth_relational::facade::publication::PublicationError {
    let WorthQueryProgramOutputAdvanceDenial::Outputs(
        WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::Demand(denial),
        ),
    ) = denial
    else {
        panic!("{denial:?}")
    };
    denial
        .authored_retirement_error()
        .unwrap_or_else(|| panic!("{denial:?}"))
        .clone()
}

#[test]
fn an_extra_authored_entity_is_not_part_of_the_owned_closure() {
    let error = refusal(Fault::Extra);
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::ExtraEntity)
    );
}

#[test]
fn an_omitted_transitive_publication_refuses_with_its_original_member_subject() {
    let error = refusal(Fault::MissingLeaf);
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::MissingPublication)
    );
}

#[test]
fn an_entity_deletion_without_a_retire_role_names_the_hidden_subject() {
    let error = refusal(Fault::HiddenDeletion);
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::ExtraEntity)
    );
}
