//! Policies cross the same MessagePack carrier as portable snapshots.
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use serde::Serialize;
use worth_signal::facade::runtime::{ObservationDeliveryMode, ObservationPolicy};

use crate::boundary::serde::{from_portable_wire, to_portable_wire};

/// The pre-Visited policy shape: an unversioned two-field struct sequence.
#[derive(Serialize)]
enum HistoricalTrigger {
    Touched,
    Recomputed,
    MeaningfulChange,
}

#[derive(Serialize)]
struct HistoricalPolicy {
    trigger: HistoricalTrigger,
    delivery_mode: ObservationDeliveryMode,
}

fn historical_policy(trigger: HistoricalTrigger) -> Vec<u8> {
    rmp_serde::to_vec(&HistoricalPolicy {
        trigger,
        delivery_mode: ObservationDeliveryMode::PerCommittedTransaction,
    })
    .unwrap()
}

#[test]
fn portable_policy_round_trips_and_refuses_historical_struct_sequences() {
    for (trigger, current) in [
        (HistoricalTrigger::Touched, ObservationPolicy::visited()),
        (
            HistoricalTrigger::Recomputed,
            ObservationPolicy::recomputed(),
        ),
        (
            HistoricalTrigger::MeaningfulChange,
            ObservationPolicy::meaningful_change(),
        ),
    ] {
        let historical = STANDARD.encode(historical_policy(trigger));
        assert!(from_portable_wire::<ObservationPolicy>(&historical).is_err());
        let current_wire = to_portable_wire(&current).unwrap();
        assert_eq!(
            from_portable_wire::<ObservationPolicy>(&current_wire).unwrap(),
            current
        );
        let sequence = rmp_serde::to_vec(&current).unwrap();
        assert_eq!(
            rmp_serde::from_slice::<ObservationPolicy>(&sequence).unwrap(),
            current
        );
    }
}

#[test]
fn portable_policy_rejects_mixed_versions_and_invalid_sequence_shapes() {
    for fields in [
        vec!["Visited", "PerCommittedTransaction"],
        vec!["Touched", "PerCommittedTransaction"],
        vec![
            "worth.signal.observation-policy.v1",
            "Touched",
            "PerCommittedTransaction",
        ],
        vec![
            "worth.signal.observation-policy.v2",
            "Touched",
            "PerCommittedTransaction",
        ],
        vec![
            "worth.signal.observation-policy.v1",
            "Visited",
            "PerCommittedTransaction",
        ],
        vec![
            "worth.signal.observation-policy.v3",
            "Recomputed",
            "PerCommittedTransaction",
        ],
        vec!["Touched"],
        vec!["Touched", "PerCommittedTransaction", "unexpected"],
        vec!["worth.signal.observation-policy.v2", "Visited"],
        vec![
            "worth.signal.observation-policy.v2",
            "Visited",
            "PerCommittedTransaction",
            "unexpected",
        ],
    ] {
        let bytes = rmp_serde::to_vec(&fields).unwrap();
        assert!(rmp_serde::from_slice::<ObservationPolicy>(&bytes).is_err());
    }
}

#[test]
fn portable_snapshot_refuses_a_historical_observation_policy() {
    use crate::expression::model::SignalValue;
    use crate::recipe::model::TransactionOp;
    use crate::runtime::core::RuntimeCore;
    use crate::runtime::policy::RuntimePolicySpec;
    use crate::runtime::summaries::RuntimeSnapshotEnvelope;

    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_web_input("counter".to_owned(), SignalValue::Number(1.0), None)
        .unwrap();
    let callback = runtime.register_native_watch_callback(Box::new(|notice| {
        assert!(notice.visited);
    }));
    let observation = runtime.watch_signal("counter", callback).unwrap();
    let edit = |value| {
        vec![TransactionOp::Set {
            id: "counter".to_owned(),
            value: SignalValue::Number(value),
            aspect: None,
            aspects: None,
        }]
    };
    runtime.apply_transaction(edit(7.0)).unwrap();
    let snapshot = runtime.snapshot().unwrap();
    assert!(snapshot.snapshot.diagnostics.latest_observation.is_some());
    let current_wire = to_portable_wire(&snapshot).unwrap();
    let current = STANDARD.decode(&current_wire).unwrap();
    let current_policy = rmp_serde::to_vec_named(&ObservationPolicy::meaningful_change()).unwrap();
    let offsets: Vec<_> = current
        .windows(current_policy.len())
        .enumerate()
        .filter_map(|(offset, bytes)| (bytes == current_policy).then_some(offset))
        .collect();
    assert_eq!(
        offsets.len(),
        2,
        "observer and committed observation each retain a policy"
    );
    // Replace only policy carriers. Runtime-issued snapshot identity and authority remain intact.
    let mut historical = Vec::new();
    let mut cursor = 0;
    for offset in offsets {
        historical.extend_from_slice(&current[cursor..offset]);
        historical.extend_from_slice(&historical_policy(HistoricalTrigger::MeaningfulChange));
        cursor = offset + current_policy.len();
    }
    historical.extend_from_slice(&current[cursor..]);
    assert!(from_portable_wire::<RuntimeSnapshotEnvelope>(&STANDARD.encode(historical)).is_err());
    let roundtrip: RuntimeSnapshotEnvelope = from_portable_wire(&current_wire).unwrap();
    assert_eq!(
        roundtrip.snapshot.diagnostics,
        snapshot.snapshot.diagnostics
    );
    runtime.apply_transaction(edit(11.0)).unwrap();
    runtime.restore_snapshot(roundtrip).unwrap();
    assert_eq!(
        runtime.read_value("counter").unwrap(),
        SignalValue::Number(7.0)
    );
    assert!(runtime.unobserve_handle(observation));
    assert!(runtime.dispose_observation_callback(callback));
}
