//! Actual field observations across absence, owner refusal and native denial.
use std::cell::Cell;

use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, AccountIdentity, AccountNote,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn absent_fields_and_predecode_refusal_observe_the_same_acquired_field() {
    let world = installed_authorization_world(true);
    let completed = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let present = reader
                            .resolve_entity(AccountIdentity::reference(), "account-1".to_owned())
                            .unwrap();
                        let absent = reader
                            .resolve_entity(AccountIdentity::reference(), "account-2".to_owned())
                            .unwrap();
                        let before = reader.work.field_reads();
                        assert_eq!(
                            reader.field(&present, AccountNote::reference()).as_deref(),
                            Some("reviewed")
                        );
                        let present_reads = reader.work.field_reads() - before;
                        let before = reader.work.field_reads();
                        assert_eq!(reader.field(&absent, AccountNote::reference()), None);
                        let absent_reads = reader.work.field_reads() - before;
                        let before = reader.work.field_reads();
                        let refused = reader.field_with_predecode_admission(
                            &present,
                            AccountNote::reference(),
                            |_, _| Err::<(), _>("owner-refused"),
                            &|| Ok(()),
                        );
                        assert_eq!(refused.unwrap(), Err("owner-refused"));
                        let refused_reads = reader.work.field_reads() - before;
                        let invoked = Cell::new(false);
                        let before = reader.work.field_reads();
                        let missing = reader.field_with_predecode_admission(
                            &absent,
                            AccountNote::reference(),
                            |_, _| {
                                invoked.set(true);
                                Ok::<_, ()>(())
                            },
                            &|| Ok(()),
                        );
                        assert_eq!(missing.unwrap(), Ok(None));
                        assert!(
                            !invoked.get(),
                            "absent values do not enter predecode admission"
                        );
                        (
                            present_reads,
                            absent_reads,
                            refused_reads,
                            reader.work.field_reads() - before,
                        )
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let &(present, absent, refused, predecode_absent) = completed.output();
    assert!(present > 0);
    assert_eq!(present, absent);
    assert_eq!(present, refused);
    assert_eq!(present, predecode_absent);
}

#[test]
fn cancelled_field_acquisition_contributes_no_field_observation() {
    let world = installed_authorization_world(true);
    let cancellation = CancellationSource::new();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&host);
    let selected = world
        .application
        .select_product_branch(
            &execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap();
    let product = selected.product().publication_binding();
    let observed = Cell::new(None);
    let denied = world.invariant.project_with_work_limit(
        &execution,
        None,
        product.observation().basis().relational_basis().clone(),
        |reader| {
            let account = reader
                .resolve_entity(AccountIdentity::reference(), "account-1".to_owned())
                .unwrap();
            let before = reader.work.field_reads();
            assert!(reader.field(&account, AccountNote::reference()).is_some());
            let acquired = reader.work.field_reads() - before;
            cancellation.cancel();
            let before = reader.work.field_reads();
            assert_eq!(reader.field(&account, AccountNote::reference()), None);
            let field = AccountNote::reference();
            let locator = reader
                .layout
                .field_locator(field.entity(), field.aspect(), field.field())
                .unwrap()
                .clone();
            let body: Result<(), _> = reader.body_scalar_observe(&account, &locator, |_| {
                panic!("the cancelled request cannot acquire the body scalar")
            });
            assert!(body.is_err());
            observed.set(Some((acquired, reader.work.field_reads() - before)));
        },
        None,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    let denial = denied
        .err()
        .expect("the cancelled projection cannot seal output");
    let (acquired, cancelled) = observed.get().expect("both reads were attempted");
    assert!(acquired > 0);
    assert_eq!(cancelled, 0);
    assert_eq!(denial.projection_work().unwrap().field_reads(), acquired);
}
