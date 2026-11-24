//! End-to-end Directory Home court for declarative `worth_query_program_transition!`,
//! zero-boilerplate `worth_query_default_build_id!()`, auto-trait contracts, and
//! 64-seed Differential DST matching `specs/tla/program_transition/ProgramTransition.tla`.

mod driver;
mod dst;
mod journey;
mod observe;
mod reference_model;
mod schema;
mod v2_mutations;
mod v2_program;
mod v2_queries;
mod v3_mutations;
mod v3_program;
mod v3_queries;

use std::panic::{RefUnwindSafe, UnwindSafe};

use worth_query_decl::facade::program_transition::{
    worth_query_default_build_id as decl_default_build_id,
    WorthQueryApplicationHomeBuildIdentity as DeclBuildId,
    WorthQueryProgramTransitionDeclarationDenial, WorthQueryProgramTransitionDescriptor,
    WorthQueryProgramTransitionFieldDefault, WorthQueryProgramTransitionIndexRebuild,
    WorthQueryProgramTransitionRetainedRecord, WorthQueryProgramTransitionSpec,
};
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::{ApplicationBuildIdentity, InvalidApplicationBuildIdentity},
    program_transition::{
        open_directory_with_transition, worth_query_default_build_id,
        WorthQueryApplicationHomeBuildIdentity, WorthQueryHomePredecessorRefusal,
        WorthQueryProgramTransitionConfigDenial, WorthQueryProgramTransitionOpenConfig,
        WorthQueryProgramTransitionOpenDenial, WorthQueryTransitionOpenOutcome,
    },
};
use worth_query_testkit_host::facade::host::{application_home as fixture, checkpoint_capture};

use v2_program::{v1_request, v2_request};
use v3_program::{v3_request, DaymarkV2ToV3, DaymarkV3ToV2Downgrade};

fn assert_auto_traits<T: Send + Sync + UnwindSafe + RefUnwindSafe>() {}

#[test]
fn program_transition_public_types_satisfy_auto_traits() {
    assert_auto_traits::<WorthQueryApplicationHomeBuildIdentity>();
    assert_auto_traits::<WorthQueryProgramTransitionDescriptor>();
    assert_auto_traits::<WorthQueryProgramTransitionFieldDefault>();
    assert_auto_traits::<WorthQueryProgramTransitionRetainedRecord>();
    assert_auto_traits::<WorthQueryProgramTransitionIndexRebuild>();
    assert_auto_traits::<WorthQueryProgramTransitionDeclarationDenial>();
    assert_auto_traits::<WorthQueryProgramTransitionOpenConfig>();
    assert_auto_traits::<WorthQueryProgramTransitionConfigDenial>();
    assert_auto_traits::<WorthQueryTransitionOpenOutcome>();
    assert_auto_traits::<WorthQueryHomePredecessorRefusal>();
    assert_auto_traits::<DaymarkV2ToV3>();
}

#[test]
fn default_build_id_macro_and_package_fallback_honor_override_and_fail_closed() {
    let from_host = worth_query_default_build_id!();
    let from_decl = decl_default_build_id!();
    assert_eq!(from_host, from_decl);
    assert_ne!(from_host.bytes(), [0u8; 16]);

    let explicit = "22222222-2222-4222-8222-222222222222";
    assert_eq!(
        WorthQueryApplicationHomeBuildIdentity::package_default(Some(explicit), "daymark@0.2.0"),
        Ok(ApplicationBuildIdentity::parse(explicit).unwrap())
    );
    assert_eq!(
        DeclBuildId::package_default(Some("invalid"), "daymark@0.2.0"),
        Err(InvalidApplicationBuildIdentity::Malformed)
    );
    assert_eq!(
        WorthQueryApplicationHomeBuildIdentity::package_default(None, ""),
        Err(InvalidApplicationBuildIdentity::Missing)
    );
    let fallback_v2 =
        WorthQueryApplicationHomeBuildIdentity::package_default(None, "daymark@0.2.0").unwrap();
    let fallback_v3 =
        WorthQueryApplicationHomeBuildIdentity::package_default(None, "daymark@0.3.0").unwrap();
    assert_ne!(fallback_v2, fallback_v3);
}

#[test]
fn fresh_v3_directory_home_reports_created_fresh_then_opened_current() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    let opened = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            open_directory_with_transition::<DaymarkV2ToV3>(
                &phase,
                v2_request(location.clone(), build),
                v3_request(location.clone(), build),
                WorthQueryProgramTransitionOpenConfig::default_bounded(),
            )
            .expect("fresh v3 directory home opens directly"),
        )
    })
    .unwrap();
    let (home, outcome) = (*opened).into_parts();
    assert_eq!(
        outcome,
        WorthQueryTransitionOpenOutcome::CreatedFresh { program_version: 3 }
    );
    home.close(&execution).unwrap().reclaim();

    let reopened = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            open_directory_with_transition::<DaymarkV2ToV3>(
                &phase,
                v2_request(location.clone(), build),
                v3_request(location, build),
                WorthQueryProgramTransitionOpenConfig::default_bounded(),
            )
            .expect("current v3 directory home reopens directly"),
        )
    })
    .unwrap();
    let (home, outcome) = (*reopened).into_parts();
    assert_eq!(
        outcome,
        WorthQueryTransitionOpenOutcome::OpenedCurrent { program_version: 3 }
    );
    home.close(&execution).unwrap().reclaim();
}

#[test]
fn wrong_predecessor_v1_refuses_with_typed_predecessor_mismatch() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    let v1_home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            v1_request(location.clone(), build)
                .open_in_advancement(&phase)
                .expect("v1 home creates"),
        )
    })
    .unwrap();
    (*v1_home).close(&execution).unwrap().reclaim();

    let refused = with_bootstrap_advancement_from_request(execution, |phase| {
        open_directory_with_transition::<DaymarkV2ToV3>(
            &phase,
            v2_request(location.clone(), build),
            v3_request(location, build),
            WorthQueryProgramTransitionOpenConfig::default_bounded(),
        )
    })
    .unwrap()
    .err()
    .expect("v1 home must not open under v2 -> v3 transition");
    assert!(
        matches!(
            refused,
            WorthQueryProgramTransitionOpenDenial::PredecessorMismatch {
                kind: WorthQueryHomePredecessorRefusal::PredecessorNotRostered
                    | WorthQueryHomePredecessorRefusal::PredecessorMismatch,
                ..
            }
        ),
        "expected typed PredecessorMismatch, got {refused:?}"
    );
}

#[test]
fn declared_downgrade_and_invalid_config_refuse_fail_closed() {
    assert_eq!(
        DaymarkV3ToV2Downgrade::descriptor(),
        Err(
            WorthQueryProgramTransitionDeclarationDenial::DowngradeRefused {
                from_version: 3,
                to_version: 2,
            }
        )
    );

    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    let downgrade_refused = with_bootstrap_advancement_from_request(execution, |phase| {
        open_directory_with_transition::<DaymarkV3ToV2Downgrade>(
            &phase,
            v3_request(location.clone(), build),
            v2_request(location.clone(), build),
            WorthQueryProgramTransitionOpenConfig::default_bounded(),
        )
    })
    .unwrap()
    .err()
    .expect("declared v3 -> v2 downgrade must fail closed");
    assert!(matches!(
        downgrade_refused,
        WorthQueryProgramTransitionOpenDenial::Declaration(
            WorthQueryProgramTransitionDeclarationDenial::DowngradeRefused {
                from_version: 3,
                to_version: 2,
            }
        )
    ));

    let bad_config = WorthQueryProgramTransitionOpenConfig::default_bounded()
        .with_max_entities_per_kind(usize::MAX);
    let config_refused = with_bootstrap_advancement_from_request(execution, |phase| {
        open_directory_with_transition::<DaymarkV2ToV3>(
            &phase,
            v2_request(location.clone(), build),
            v3_request(location, build),
            bad_config,
        )
    })
    .unwrap()
    .err()
    .expect("usize::MAX scan limit must refuse");
    assert!(matches!(
        config_refused,
        WorthQueryProgramTransitionOpenDenial::InvalidConfiguration(
            WorthQueryProgramTransitionConfigDenial::InvalidEntityScanLimit
        )
    ));
}
