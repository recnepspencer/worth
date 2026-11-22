//! A Native source stop keeps its original request disposition through Query.
use super::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Interruption;
use worth_relational::facade::{
    runtime::{
        CustomInvariantInterruption as Stop, DerivedRelationReadDenial as Read,
        ManagedIndexSourcePreparationDenial as Native, StructuralWalkDenialKind as Structural,
    },
    transactions::RelationalExecutionDenialCause as Execution,
};

#[test]
fn native_source_interruptions_survive_the_descriptive_boundary() {
    for (stop, expected) in [
        (Stop::Cancelled, Interruption::Cancelled),
        (Stop::DeadlineElapsed, Interruption::DeadlineExceeded),
    ] {
        let denial = Native::Source(Read::Structural(Structural::Interrupted(stop)));
        assert_eq!(native_source(&denial), Cause::Interrupted(expected));
    }
    let denial = Native::Source(Read::Request(Execution::Cancelled));
    assert_eq!(
        native_source(&denial),
        Cause::Execution(&Execution::Cancelled)
    );
    let denial = Native::Publication(
        worth_relational::facade::mvcc::CompanionPreflightStop::Execution(
            Execution::DeadlineElapsed,
        ),
    );
    assert!(matches!(
        native_source(&denial),
        Cause::NativeAdmission(
            worth_relational::facade::mvcc::CompanionPreflightStop::Execution(
                Execution::DeadlineElapsed
            )
        )
    ));
}
