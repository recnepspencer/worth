use std::cell::Cell;
use std::num::NonZeroUsize;
use worth_execution::CancellationSource;
use worth_execution::CancellationToken;
use worth_execution::ExecutionRequest;
use worth_execution::ExecutionWorkCeiling;
use worth_execution::MapKernelStop;
use worth_execution::SerialRequest;
use worth_execution::WorkCeilingDenial;
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionBudget;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::value_decode::state::ValueDecodeState;
use worth_query_execution::facade::domain_computation::WorthQueryValueDecodeDenial as Stop;

pub fn native_request(cancellation: CancellationToken) -> SerialRequest {
    SerialRequest::from_policy(
        &ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            NonZeroUsize::MIN,
        ),
        cancellation,
        None,
    )
}

#[test]
fn decode_scope_keeps_worker_only_native_and_preserves_an_accounted_ancestor() {
    let backing = native_request(CancellationToken::new());
    let request = ExecutionRequest::serial(&backing);
    let native = ValueDecodeState::new(Default::default());
    native
        .run_for_test(&request, |session| {
            assert!((*session.checkpoint_for_test()).is_none());
            assert!((*native.memory_for_test()).borrow().is_none());
        })
        .unwrap();

    let accounted = ValueDecodeState::new(Default::default());
    let parent = SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, None),
        ),
        CancellationToken::new(),
        None,
    );
    ExecutionRequest::serial(&parent)
        .run(ExecutionWorkCeiling::new(None), |_| {
            accounted
                .run_for_test(&request, |session| {
                    assert!((*session.checkpoint_for_test()).is_some());
                    assert!((*accounted.memory_for_test()).borrow().is_some());
                })
                .unwrap();
        })
        .unwrap();
}

#[test]
fn cancelled_native_decode_stops_before_body_and_keeps_its_original_refusal() {
    let cancellation = CancellationSource::new();
    let backing = native_request(cancellation.token());
    let request = ExecutionRequest::serial(&backing);
    let state = ValueDecodeState::new(Default::default());
    let called = Cell::new(false);
    cancellation.cancel();
    let expected = Stop::Scope(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled));
    assert_eq!(
        state.run_for_test(&request, |_| called.set(true)),
        Err(expected)
    );
    assert!(!called.get());
    assert_eq!(state.denial_for_test(), Some(expected));
    assert_eq!(
        state.run_for_test(&request, |_| called.set(true)),
        Err(expected)
    );
    assert!(!called.get());
    assert!((*state.memory_for_test()).borrow().is_none());
}

#[test]
fn a_record_decoder_catching_original_cancellation_cannot_return_a_value() {
    let cancellation = CancellationSource::new();
    let backing = native_request(cancellation.token());
    let request = ExecutionRequest::serial(&backing);
    let state = ValueDecodeState::new(Default::default());
    let immediate = Cell::new(None);
    let denied = state.run_for_test(&request, |session| {
        let decoded = session.with_record_admission_for_test(|admission| {
            cancellation.cancel();
            assert!(admission.admit(1, 0).is_err());
            17
        });
        immediate.set(decoded.err());
    });
    let original = Stop::Checkpoint(MapKernelStop::Cancelled);
    assert_eq!(immediate.get(), Some(original));
    assert!(denied.is_err());
    assert_eq!(state.denial_for_test(), Some(original));
}

use std::sync::atomic::{AtomicUsize, Ordering};
use worth_foundational::facade::{AspectValue, InlineBytes, InternedString, ScalarAspectType};
use worth_query_declaration::facade::application_schema::{
    ApplicationNativeInputDecodeProfile, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationValueDecodeAvailable, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial, ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial,
    InlineBytesApplicationValueBinding, StringApplicationValueBinding, ValueDecodeBounds,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog;

static FALLBACK_DECODE_CALLS: AtomicUsize = AtomicUsize::new(0);
static MISSING_PROFILE_DECODE_CALLS: AtomicUsize = AtomicUsize::new(0);

struct DefaultFallbackNativeBinding;
impl ApplicationScalarValueBinding for DefaultFallbackNativeBinding {
    type Value = String;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.test.value-decode.native-fallback.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    const SUPPORTS_ADMITTED_DECODE: bool = true;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        Some(ApplicationNativeInputDecodeProfile::new(50, 2, 24, 1));
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::String(value.clone().into()))
    }
}
impl ApplicationReadableScalarValueBinding for DefaultFallbackNativeBinding {
    // Deliberately inherits the default `decode_admitted` implementation.
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        FALLBACK_DECODE_CALLS.fetch_add(1, Ordering::SeqCst);
        StringApplicationValueBinding::decode(value)
    }
}

struct MissingProfileNativeBinding;
impl ApplicationScalarValueBinding for MissingProfileNativeBinding {
    type Value = u64;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.test.value-decode.missing-profile.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::UInt64;
    const SUPPORTS_ADMITTED_DECODE: bool = true;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> = None;
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::UInt64(*value))
    }
}
impl ApplicationReadableScalarValueBinding for MissingProfileNativeBinding {
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        MISSING_PROFILE_DECODE_CALLS.fetch_add(1, Ordering::SeqCst);
        match value {
            AspectValue::UInt64(n) => Ok(*n),
            other => Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
                binding_identity: Self::IDENTITY,
                expected: Self::SCALAR_FAMILY,
                observed: other.value_family(),
            }),
        }
    }
}

worth_query_declaration::worth_query_value_binding!(SessionCanonicalProbeBinding for u64 {
    identity: "worth.test.value-decode.session-canonical-probe.v1",
    scalar: String,
    canonical: ValueDecodeBounds::new(64, 32, 4, 4),
});

fn bounded_probe_request(memory_limit: Option<u64>, work_limit: Option<u64>) -> SerialRequest {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, memory_limit, work_limit),
    );
    SerialRequest::from_policy(&policy, CancellationSource::new().token(), None)
}

#[test]
fn native_string_and_inline_bytes_probes_charge_per_byte_profile_before_decode() {
    let payload = "a".repeat(4096);
    let raw_string = AspectValue::String(InternedString::from(payload.clone()));
    let tight = bounded_probe_request(Some(64 * 1024), Some(100));
    let state =
        ValueDecodeState::admitted(WorthQueryInstalledApplicationValueBindingCatalog::default());
    let refused = state
        .run(&ExecutionRequest::serial(&tight), |session| {
            session.decode_managed_probe::<StringApplicationValueBinding>(&raw_string)
        })
        .unwrap_err();
    assert!(matches!(
        refused,
        Stop::Checkpoint(MapKernelStop::WorkCeiling)
    ));

    let low_mem = bounded_probe_request(Some(2048), Some(10_000));
    let state =
        ValueDecodeState::admitted(WorthQueryInstalledApplicationValueBindingCatalog::default());
    let mem_refused = state
        .run(&ExecutionRequest::serial(&low_mem), |session| {
            session.decode_managed_probe::<StringApplicationValueBinding>(&raw_string)
        })
        .unwrap_err();
    assert!(matches!(mem_refused, Stop::Memory(_)));

    let enough = bounded_probe_request(Some(64 * 1024), Some(10_000));
    let state =
        ValueDecodeState::admitted(WorthQueryInstalledApplicationValueBindingCatalog::default());
    let decoded = state
        .run(&ExecutionRequest::serial(&enough), |session| {
            session.decode_managed_probe::<StringApplicationValueBinding>(&raw_string)
        })
        .unwrap();
    assert_eq!(decoded, Some(payload));
    assert!(enough.memory().unwrap().charged_memory_bytes() >= 4096);

    let raw_bytes = AspectValue::InlineBytes(InlineBytes::try_new(vec![7u8; 2048]).unwrap());
    let tight_bytes = bounded_probe_request(Some(64 * 1024), Some(50));
    let state =
        ValueDecodeState::admitted(WorthQueryInstalledApplicationValueBindingCatalog::default());
    let bytes_refused = state
        .run(&ExecutionRequest::serial(&tight_bytes), |session| {
            session.decode_managed_probe::<InlineBytesApplicationValueBinding>(&raw_bytes)
        })
        .unwrap_err();
    assert!(matches!(
        bytes_refused,
        Stop::Checkpoint(MapKernelStop::WorkCeiling)
    ));
}

#[test]
fn native_binding_inheriting_decode_admitted_obeys_native_profile_and_missing_profile_refuses() {
    FALLBACK_DECODE_CALLS.store(0, Ordering::SeqCst);
    let raw = AspectValue::String(InternedString::from("hello-native-probe".to_owned()));
    let tight = bounded_probe_request(Some(64 * 1024), Some(40));
    let state =
        ValueDecodeState::admitted(WorthQueryInstalledApplicationValueBindingCatalog::default());
    let refused = state
        .run(&ExecutionRequest::serial(&tight), |session| {
            session.decode_managed_probe::<DefaultFallbackNativeBinding>(&raw)
        })
        .unwrap_err();
    assert!(matches!(
        refused,
        Stop::Checkpoint(MapKernelStop::WorkCeiling)
    ));
    assert_eq!(FALLBACK_DECODE_CALLS.load(Ordering::SeqCst), 0);

    let enough = bounded_probe_request(Some(64 * 1024), Some(200));
    let state =
        ValueDecodeState::admitted(WorthQueryInstalledApplicationValueBindingCatalog::default());
    let decoded = state
        .run(&ExecutionRequest::serial(&enough), |session| {
            session.decode_managed_probe::<DefaultFallbackNativeBinding>(&raw)
        })
        .unwrap();
    assert_eq!(decoded.as_deref(), Some("hello-native-probe"));
    assert_eq!(FALLBACK_DECODE_CALLS.load(Ordering::SeqCst), 1);

    MISSING_PROFILE_DECODE_CALLS.store(0, Ordering::SeqCst);
    let state =
        ValueDecodeState::admitted(WorthQueryInstalledApplicationValueBindingCatalog::default());
    let missing = state
        .run(&ExecutionRequest::serial(&enough), |session| {
            session.decode_managed_probe::<MissingProfileNativeBinding>(&AspectValue::UInt64(9))
        })
        .unwrap_err();
    assert!(matches!(missing, Stop::CodecRejected));
    assert_eq!(MISSING_PROFILE_DECODE_CALLS.load(Ordering::SeqCst), 0);
}

#[test]
fn repeat_native_probe_decodes_exhaust_bounded_work_ceiling() {
    let raw = AspectValue::String(InternedString::from("r".repeat(512)));
    let request = bounded_probe_request(Some(64 * 1024), Some(1200));
    let state =
        ValueDecodeState::admitted(WorthQueryInstalledApplicationValueBindingCatalog::default());
    let mut completed = 0usize;
    let refused = state
        .run(&ExecutionRequest::serial(&request), |session| {
            for _ in 0..5 {
                if session
                    .decode_managed_probe::<StringApplicationValueBinding>(&raw)
                    .is_some()
                {
                    completed += 1;
                } else {
                    break;
                }
            }
        })
        .unwrap_err();
    assert_eq!(completed, 2);
    assert!(matches!(
        refused,
        Stop::Checkpoint(MapKernelStop::WorkCeiling)
    ));
}

#[test]
fn canonical_probe_keeps_callback_admission_path_and_refuses_zero_budget() {
    assert!(SessionCanonicalProbeBinding::NATIVE_INPUT_ADMISSION.is_none());
    assert!(StringApplicationValueBinding::NATIVE_INPUT_ADMISSION.is_some());

    let raw = AspectValue::String(InternedString::from("1234567890".to_owned()));
    let zero_work = bounded_probe_request(Some(64 * 1024), Some(0));
    let state =
        ValueDecodeState::admitted(WorthQueryInstalledApplicationValueBindingCatalog::default());
    let refused = state
        .run(&ExecutionRequest::serial(&zero_work), |session| {
            session.decode_managed_probe::<SessionCanonicalProbeBinding>(&raw)
        })
        .unwrap_err();
    assert!(matches!(
        refused,
        Stop::Checkpoint(MapKernelStop::WorkCeiling)
    ));
    assert_eq!(zero_work.memory().unwrap().charged_memory_bytes(), 0);

    let native_zero = bounded_probe_request(Some(64 * 1024), Some(0));
    let native_state =
        ValueDecodeState::admitted(WorthQueryInstalledApplicationValueBindingCatalog::default());
    let native_refused = native_state
        .run(&ExecutionRequest::serial(&native_zero), |session| {
            session.decode_managed_probe::<StringApplicationValueBinding>(&raw)
        })
        .unwrap_err();
    assert!(matches!(
        native_refused,
        Stop::Checkpoint(MapKernelStop::WorkCeiling)
    ));
    assert_eq!(native_zero.memory().unwrap().charged_memory_bytes(), 0);

    let enough = bounded_probe_request(Some(64 * 1024), Some(200));
    let state =
        ValueDecodeState::admitted(WorthQueryInstalledApplicationValueBindingCatalog::default());
    let decoded = state
        .run(&ExecutionRequest::serial(&enough), |session| {
            session.decode_managed_probe::<SessionCanonicalProbeBinding>(&raw)
        })
        .unwrap();
    assert_eq!(decoded, Some(1_234_567_890));
}
