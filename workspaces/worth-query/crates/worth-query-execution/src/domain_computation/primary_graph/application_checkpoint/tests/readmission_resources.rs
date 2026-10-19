//! Actual bootstrap payer at Query's metadata boundary. Literal native bytes
//! prove neither native recovery nor published-runtime authority.
use super::{accepted_with_raw_role, checkpoint_body, checkpoint_from_body, entity_bytes};
use crate::domain_computation::primary_graph::{
    application_contribution::with_bootstrap_advancement,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};

#[test]
fn bootstrap_metadata_keeps_its_charge_and_refuses_an_unfunded_row() {
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(1 << 20), Some(1_000_000)),
    );
    with_bootstrap_advancement(policy, |phase| {
        let execution = phase.execution_request();
        let worth_execution::ExecutionAllocationPolicy::Execution(lease) =
            execution.allocation_policy()
        else {
            panic!("bootstrap must supply its admitted lease")
        };
        let mut accepted = accepted_with_raw_role(b"subject", 0, b"Entity", entity_bytes());
        let source = super::super::facts::encode(&[
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceEntity {
                entity_id: worth_relational::facade::identity::EntityId::new(
                    worth_relational::facade::identity::PartitionId::main(), 1, 1),
            }]).unwrap();
        accepted.truncate(accepted.len() - 10);
        accepted.extend_from_slice(&super::super::facts::WIRE_VERSION.to_be_bytes());
        accepted.extend_from_slice(&(source.len() as u64).to_be_bytes());
        accepted.extend_from_slice(&source);
        let checkpoint = checkpoint_from_body(checkpoint_body(1, accepted));
        let original = checkpoint.bytes().to_vec();
        let before = lease.charged_memory_bytes();
        let decoded = checkpoint
            .clone()
            .decode_with_execution(&execution)
            .unwrap();
        assert!(
            lease.charged_memory_bytes() > before,
            "decoded metadata retains its reservation beyond decode scope"
        );
        let recovered = crate::domain_computation::primary_graph::application_output_demand::WorthQueryRecoveredOutputs::from_checkpoint(
            decoded, &execution, |accepted| crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
                crate::equality_type_id::EqualityTypeId::of::<()>(),
                crate::equality_type_id::EqualityTypeId::of::<()>(), std::iter::empty(), &accepted.roles, |_| Some(crate::equality_type_id::EqualityTypeId::of::<()>()), &execution)
                .map(std::sync::Arc::new)).unwrap();
        let escaped = recovered.iter().next().unwrap().clone();
        let twin = escaped.clone();
        assert!(std::sync::Arc::ptr_eq(&escaped.checkpoint.producer, &twin.checkpoint.producer));
        assert!(std::sync::Arc::ptr_eq(&escaped.checkpoint.roles, &twin.checkpoint.roles));
        assert!(std::sync::Arc::ptr_eq(escaped.checkpoint.producer_facts.as_ref().unwrap(),
            twin.checkpoint.producer_facts.as_ref().unwrap()));
        assert_eq!(twin.checkpoint.producer_facts.as_ref().unwrap().as_slice(), source);
        drop(recovered);
        drop(escaped);
        assert!(lease.charged_memory_bytes() > before,
            "the last escaped identity and correspondence keep their independent payers after runtime metadata drops");
        assert_eq!(twin.checkpoint.producer.as_str(), "producer");
        drop(twin);
        assert_eq!(lease.charged_memory_bytes(), before);

        let held = lease
            .reserve_memory(lease.available_memory_bytes().expect("the proof declares a finite memory limit"))
            .unwrap();
        // Header/current framing already has a valid positive control above.
        // Enter the same current-body parser directly so scope-framework memory
        // cannot counterfeit the row allocation refusal this case protects.
        let denial = checkpoint
            .clone()
            .decode_current_body(&execution)
            .err()
            .expect("a row cannot allocate before its bootstrap payer admits it");
        use crate::domain_computation::{
            primary_graph::application_contribution::{
                WorthQueryManagedComputationResourceDenial as Resource,
                WorthQueryMemoryLimitLevel as Level,
            },
            WorthQueryProviderSessionDenialKind as Provider,
        };
        assert!(
            matches!(denial.into_installation_denial().kind(), Kind::ExecutionDenied {
            kind: Provider::ExecutionResource {
                denial: Resource::MemoryLimit { requested, admitted, level: Level::Policy }, ..
            }
        } if requested > admitted)
        );
        assert_eq!(checkpoint.bytes(), original);
        drop(held);
        let retry = checkpoint.decode_with_execution(&execution).unwrap();
        assert_eq!(retry.accepted_outputs.len(), 1);
    })
    .unwrap();
}

#[test]
fn actor_fact_execution_causes_remain_native_at_bootstrap() {
    use super::super::{CheckpointDecodeDenial, FactDecodeDenial};
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
    for (cause, expected) in [
        (Native::Cancelled, Control::Cancelled),
        (Native::DeadlineElapsed, Control::TimedOut),
    ] {
        let denial = CheckpointDecodeDenial::from_fact(FactDecodeDenial::Execution(cause))
            .into_installation_denial();
        assert!(
            matches!(denial.kind(), Kind::ExecutionControlStopped { kind } if kind == expected)
        );
    }
    let denial = CheckpointDecodeDenial::from_fact(FactDecodeDenial::Execution(
        Native::PolicyMemoryExhausted {
            requested: 37,
            admitted: 11,
            ancestor: 0,
        },
    ))
    .into_installation_denial();
    use crate::domain_computation::{
        primary_graph::application_contribution::{
            WorthQueryManagedComputationResourceDenial as Resource,
            WorthQueryMemoryLimitLevel as Level,
        },
        WorthQueryProviderSessionDenialKind as Provider,
    };
    assert!(matches!(
        denial.kind(),
        Kind::ExecutionDenied {
            kind: Provider::ExecutionResource {
                denial: Resource::MemoryLimit {
                    requested: 37,
                    admitted: 11,
                    level: Level::Policy
                },
                ..
            }
        }
    ));
}
