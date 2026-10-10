use super::*;
use crate::harness::fixtures::BridgeHarnessFixture;
use crate::source::{
    AdmittedSourceContract, SourceDeclarationIdentity, SourceFailureRecord,
    SourceMaterializationRecord,
};

mod certification_bundle;
pub(in crate::harness::adapter::adapter_impl) mod terminal_report_export;
#[cfg(test)]
mod typed_certification_tests;

pub(super) enum SourceHarnessTarget {
    Materialize {
        declaration_identity: SourceDeclarationIdentity,
    },
    MaterializeBatch {
        declaration_identity: SourceDeclarationIdentity,
    },
    Replay {
        declaration_identity: SourceDeclarationIdentity,
    },
    RejectUnregistered {
        declaration_identity: SourceDeclarationIdentity,
    },
    RejectOpenSnapshot {
        declaration_identity: SourceDeclarationIdentity,
    },
    RejectSnapshotDrift {
        declaration_identity: SourceDeclarationIdentity,
    },
}

pub(super) enum SourceHarnessExecution {
    Materialize {
        contract: AdmittedSourceContract,
        record: SourceMaterializationRecord,
    },
    Replay {
        contract: AdmittedSourceContract,
        record: SourceMaterializationRecord,
        replayed: SourceMaterializationRecord,
    },
    Rejected {
        failure: SourceFailureRecord,
    },
}

pub(super) fn execute_source_request(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    target: SourceHarnessTarget,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<SourceHarnessExecution, BridgeHarnessError> {
    match target {
        SourceHarnessTarget::Materialize {
            declaration_identity,
        } => {
            let (contract, record) = materialize_source_record(
                runtime_bridge,
                fixture,
                &declaration_identity,
                execution,
            )?;
            Ok(SourceHarnessExecution::Materialize { contract, record })
        }
        SourceHarnessTarget::MaterializeBatch {
            declaration_identity,
        } => {
            let (contract, record) = materialize_source_batch_record(
                runtime_bridge,
                fixture,
                &declaration_identity,
                execution,
            )?;
            Ok(SourceHarnessExecution::Materialize { contract, record })
        }
        SourceHarnessTarget::Replay {
            declaration_identity,
        } => {
            let (contract, record) = materialize_source_record(
                runtime_bridge,
                fixture,
                &declaration_identity,
                execution,
            )?;
            let replayed = runtime_bridge
                .replay_source_materialization_record(&record, execution)
                .map_err(|error| {
                    BridgeHarnessError::new(format!("bridge source replay failed: {error}"))
                })?;
            Ok(SourceHarnessExecution::Replay {
                contract,
                record,
                replayed,
            })
        }
        SourceHarnessTarget::RejectUnregistered {
            declaration_identity,
        } => reject_unregistered_source(runtime_bridge, fixture, declaration_identity),
        SourceHarnessTarget::RejectOpenSnapshot {
            declaration_identity,
        } => reject_source_materialization(
            runtime_bridge,
            fixture,
            &declaration_identity,
            |kind| matches!(kind, crate::error::BridgeDeliveryErrorKind::SnapshotAcquisitionFailure(cause) if cause.kind() == crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure),
            execution,
        ),
        SourceHarnessTarget::RejectSnapshotDrift {
            declaration_identity,
        } => reject_source_materialization(
            runtime_bridge,
            fixture,
            &declaration_identity,
            |kind| {
                matches!(
                    kind,
                    crate::error::BridgeDeliveryErrorKind::SnapshotIdentityMismatch
                )
            },
            execution,
        ),
    }
}

fn materialize_source_record(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: &SourceDeclarationIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(AdmittedSourceContract, SourceMaterializationRecord), BridgeHarnessError> {
    let declaration = fixture
        .source_declarations()
        .iter()
        .find(|declaration| declaration.declaration_identity() == declaration_identity)
        .cloned()
        .ok_or_else(|| {
            BridgeHarnessError::new(format!(
                "bridge source fixture does not declare source `{}`",
                declaration_identity.as_str()
            ))
        })?;
    let contract = runtime_bridge.admit_source(declaration).map_err(|error| {
        BridgeHarnessError::new(format!("bridge source admission failed: {error}"))
    })?;
    let observation = runtime_bridge
        .materialize_source_packet(&contract, SnapshotReadPacket::new(vec![]), execution)
        .map_err(|error| {
            BridgeHarnessError::new(format!("bridge source materialization failed: {error}"))
        })?;
    let record = runtime_bridge
        .canonicalize_source_materialization_record(&contract, &observation)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge source materialization canonicalization failed: {error}"
            ))
        })?;
    Ok((contract, record))
}

fn materialize_source_batch_record(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: &SourceDeclarationIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(AdmittedSourceContract, SourceMaterializationRecord), BridgeHarnessError> {
    let declaration = fixture
        .source_declarations()
        .iter()
        .find(|declaration| declaration.declaration_identity() == declaration_identity)
        .cloned()
        .ok_or_else(|| {
            BridgeHarnessError::new(format!(
                "bridge source fixture does not declare source `{}`",
                declaration_identity.as_str()
            ))
        })?;
    let contract = runtime_bridge.admit_source(declaration).map_err(|error| {
        BridgeHarnessError::new(format!("bridge source admission failed: {error}"))
    })?;
    let materialized = runtime_bridge
        .materialize_source_packet_batch(
            &contract,
            vec![
                SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
                    "entity-1",
                    crate::snapshot::SnapshotReadContract::scalar(
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid snapshot aspect key"),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                )]),
                SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
                    "entity-2",
                    crate::snapshot::SnapshotReadContract::scalar(
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid snapshot aspect key"),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                )]),
            ],
            execution,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge source batch materialization failed: {error}"
            ))
        })?;
    let record = runtime_bridge
        .canonicalize_source_materialization_packet_set_record(&materialized)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge source batch materialization canonicalization failed: {error}"
            ))
        })?;
    Ok((contract, record))
}

fn reject_unregistered_source(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: SourceDeclarationIdentity,
) -> Result<SourceHarnessExecution, BridgeHarnessError> {
    let template = fixture.source_declarations().first().ok_or_else(|| {
        BridgeHarnessError::new("bridge source fixture requires a source declaration")
    })?;
    let declaration = crate::source::SourceDeclaration::new(
        declaration_identity.clone(),
        template.selector().clone(),
        template.required_capabilities().clone(),
    );
    let error = runtime_bridge
        .admit_source(declaration)
        .expect_err("hostile source harness request should be rejected");
    if error.kind() != crate::error::BridgeDeliveryErrorKind::SourceContractMismatch {
        return Err(BridgeHarnessError::new(format!(
            "hostile source rejection yielded unexpected error kind `{:?}`",
            error.kind()
        )));
    }
    let failure = runtime_bridge
        .diagnostics()
        .source_failure_for_declaration_identity(declaration_identity.as_str())
        .ok_or_else(|| {
            BridgeHarnessError::new(format!(
                "bridge runtime did not retain canonical source failure for `{}`",
                declaration_identity.as_str()
            ))
        })?;

    Ok(SourceHarnessExecution::Rejected { failure })
}

fn reject_source_materialization(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: &SourceDeclarationIdentity,
    expected_error_kind: fn(&crate::error::BridgeDeliveryErrorKind) -> bool,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<SourceHarnessExecution, BridgeHarnessError> {
    let declaration = fixture
        .source_declarations()
        .iter()
        .find(|declaration| declaration.declaration_identity() == declaration_identity)
        .cloned()
        .ok_or_else(|| {
            BridgeHarnessError::new(format!(
                "bridge source fixture does not declare source `{}`",
                declaration_identity.as_str()
            ))
        })?;
    let contract = runtime_bridge.admit_source(declaration).map_err(|error| {
        BridgeHarnessError::new(format!("bridge source admission failed: {error}"))
    })?;
    let error = match runtime_bridge.materialize_source_packet(
        &contract,
        SnapshotReadPacket::new(vec![]),
        execution,
    ) {
        Ok(_) => panic!("hostile source materialization should fail"),
        Err(error) => error,
    };
    if !expected_error_kind(&error.kind()) {
        return Err(BridgeHarnessError::new(format!(
            "hostile source materialization yielded unexpected error kind `{:?}`",
            error.kind()
        )));
    }
    let failure = runtime_bridge
        .diagnostics()
        .source_failure_for_declaration_identity(declaration_identity.as_str())
        .ok_or_else(|| {
            BridgeHarnessError::new(format!(
                "bridge runtime did not retain canonical source failure for `{}`",
                declaration_identity.as_str()
            ))
        })?;
    Ok(SourceHarnessExecution::Rejected { failure })
}
