//! A retired source may read a current external parent without admitting its writer.
use super::*;
use crate::certification::document_history::owned_outputs::authored_member as member;
use worth_relational::facade::publication::AuthoredRetirementRefusal;

fn exercise(current: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = host(None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut original = None;
    if current {
        let mut demand = request
            .demand(first::Demand("a".into()))
            .start_in_program::<Program, ModelTree>(&host)
            .unwrap();
        loop {
            match demand.settle(&request).unwrap() {
                WorthQueryApplicationOutputDemandProgress::Settled(output) => {
                    drop(output);
                    break;
                }
                WorthQueryApplicationOutputDemandProgress::Progressed
                | WorthQueryApplicationOutputDemandProgress::Pending => {}
            }
        }
        demand.close();
        original = Some(
            request
                .query(query::first_output::Read("first:a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .clone(),
        );
    }
    let before = super::super::super::courts::native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency(format!("retire-with-parent-{current}"))
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<ModelTree, _>(member::retire::Intent)
        .unwrap();
    let (outcome, work) = observe_maintenance_work(8, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    if current {
        assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
        assert_eq!(work.published_rows.len() + work.omitted_publications, 1);
        assert!(request
            .query(query::member_public::Read("m".into()))
            .execute()
            .is_err());
        let row = request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap();
        let original = original.unwrap();
        assert_eq!(row.rows()[0].native_id, original.native_id);
        assert_eq!((row.rows()[0].value, original.value), (10, 10));
    } else {
        let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial)) = outcome else {
            panic!("{outcome:?}");
        };
        assert_eq!(
            denial
                .authored_retirement_error()
                .unwrap()
                .authored_retirement_refusal(),
            Some(AuthoredRetirementRefusal::ExtraCoupledOwner)
        );
        assert_eq!(work.published_rows.len() + work.omitted_publications, 0);
        assert_eq!(super::super::super::courts::native_head(&host), before);
        assert_eq!(
            request
                .query(query::member_public::Read("m".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .key,
            "m"
        );
    }
}
#[test]
fn retirement_reads_a_current_parent_outside_its_writer_catalog() {
    exercise(true);
}
#[test]
fn retirement_cannot_create_a_parent_outside_its_writer_catalog() {
    exercise(false);
}
