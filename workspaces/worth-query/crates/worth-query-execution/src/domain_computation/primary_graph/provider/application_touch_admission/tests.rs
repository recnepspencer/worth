//! Native candidate and installed grant controls; no fabricated proof inputs.
use super::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::PortableAspectPatchOperation as Op;
use worth_relational::facade::{
    mvcc::{RelationalTransactionIntent, ValidatedRelationalProposal},
    runtime::RelationalRuntime,
    transactions::*,
};
mod fixture;
use fixture::World;

fn check(run: impl FnOnce(&ExecutionRequest<'_, '_>)) {
    let host = crate::test_request_policy::serial_request();
    run(&ExecutionRequest::serial(&host));
}
#[test]
fn optional_struct_initialization_uses_only_actual_field_grants() {
    check(|execution| {
        let world = World::new(execution, false, &[]);
        let candidate = world
            .candidate(execution, [world.set(&[("A", "initial")])])
            .unwrap();
        let projection = candidate.mutation_touches(execution).unwrap();
        assert_eq!(
            projection.struct_touches().len(),
            2,
            "the SDK also writes its structural presence scalar"
        );
        let actual = projection
            .struct_touches()
            .iter()
            .find(|touch| touch.field().as_str() == "A")
            .unwrap();
        assert!(!actual.before_present() && actual.after_present());
        assert_eq!(actual.field().as_str(), "A");
        world
            .admit(execution, &candidate, false)
            .expect("B is not implicitly required or granted");
    });
}
#[test]
fn whole_replacement_requires_grants_for_omitted_prior_fields() {
    check(|execution| {
        let world = World::new(execution, false, &[("A", "equal"), ("B", "protected")]);
        let before = world
            .native
            .observe_branch(&world.native.main_branch_identity())
            .unwrap()
            .0;
        let payload = world.truth(execution);
        let candidate = world
            .candidate(execution, [world.set(&[("A", "equal")])])
            .unwrap();
        assert!(matches!(
            world.admit(execution, &candidate, false),
            Err(TouchAdmissionFailure::Contract)
        ));
        world
            .admit(execution, &candidate, true)
            .expect("actual complete grants admit removal and equal rewrite");
        assert_eq!(
            world
                .native
                .observe_branch(&world.native.main_branch_identity())
                .unwrap()
                .0,
            before
        );
        assert_eq!(world.truth(execution), payload);
    });
}
#[test]
fn whole_clear_requires_explicit_aspect_removal_grant() {
    check(|execution| {
        let world = World::new(execution, false, &[("A", "sentinel"), ("B", "protected")]);
        let before = world
            .native
            .observe_branch(&world.native.main_branch_identity())
            .unwrap()
            .0;
        let payload = world.truth(execution);
        let candidate = world.candidate(execution, [world.clear()]).unwrap();
        let projected = candidate.mutation_touches(execution).unwrap();
        assert!(projected
            .struct_touches()
            .iter()
            .all(|t| t.before_present() && !t.after_present()));
        assert!(matches!(
            world.admit(execution, &candidate, false),
            Err(TouchAdmissionFailure::Contract)
        ));
        let fields = world
            .schema
            .installed_operation(fixture::FieldsOnly::reference())
            .unwrap();
        let contracts = fields.contracts();
        assert!(
            matches!(
                admit_validated_application_touches(
                    execution,
                    &candidate,
                    &world.layout,
                    contracts.graph_reads(),
                    contracts.touches(),
                    contracts.read_touch_overlap(),
                ),
                Err(TouchAdmissionFailure::Contract)
            ),
            "all field writes cannot substitute for explicit parent removal"
        );
        world
            .admit(execution, &candidate, true)
            .expect("complete actual removal grants");
        assert_eq!(
            world
                .native
                .observe_branch(&world.native.main_branch_identity())
                .unwrap()
                .0,
            before
        );
        assert_eq!(world.truth(execution), payload);
    });
}
mod whole_sequence;
#[test]
fn sdk_record_presence_requires_its_derived_field_write_grant() {
    check(|execution| {
        let world = World::new(execution, false, &[]);
        let before = world
            .native
            .observe_branch(&world.native.main_branch_identity())
            .unwrap()
            .0;
        let candidate = world.candidate(execution, [world.set(&[])]).unwrap();
        assert!(matches!(
            world.admit_read_only(execution, &candidate),
            Err(TouchAdmissionFailure::Contract)
        ));
        world
            .admit(execution, &candidate, true)
            .expect("record field writes explicitly derive the SDK presence grant");
        assert_eq!(
            world
                .native
                .observe_branch(&world.native.main_branch_identity())
                .unwrap()
                .0,
            before
        );
    });
}
#[test]
fn required_present_replacement_is_distinct_from_native_clear_refusal() {
    check(|execution| {
        let world = World::new(execution, true, &[("A", "present"), ("B", "protected")]);
        let candidate = world
            .candidate(execution, [world.set(&[("A", "replacement")])])
            .unwrap();
        world
            .admit(execution, &candidate, true)
            .expect("Required present-to-present replacement with complete grants");
        let before = world
            .native
            .observe_branch(&world.native.main_branch_identity())
            .unwrap()
            .0;
        let denied = world.candidate(execution, [world.clear()]).unwrap_err();
        let TransactionCommitError::Conflict { error, .. } = denied else {
            panic!("native Required clear must refuse at readmission");
        };
        assert!(matches!(
            error.class,
            ConflictClass::RecordAspectPatchDenied { .. }
        ));
        let detail = format!("{:?}", error.class);
        assert!(
            detail.contains("WholeClearDenied") && detail.contains("Required"),
            "original native absence-law cause: {detail}"
        );
        assert_eq!(
            world
                .native
                .observe_branch(&world.native.main_branch_identity())
                .unwrap()
                .0,
            before
        );
    });
}
mod request_stops;

mod relation_initialization;
