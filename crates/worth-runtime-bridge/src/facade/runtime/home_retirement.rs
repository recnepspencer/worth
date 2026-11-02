//! Retain configuration while consuming each authentic Native source alias.
use super::*;
use std::{any::Any, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    change_source::RelationalRuntimeHandleAliasRetirementDenial, runtime::RelationalRuntime,
};

#[derive(Debug)]
pub enum BridgeHomeRetirementDenial {
    Busy,
    Unsupported,
    ForeignOwner,
    NativeAlias(RelationalRuntimeHandleAliasRetirementDenial),
    Execution(worth_execution::WorkCeilingDenial),
    Signal(worth_signal::facade::runtime::SignalInitialEmptyStopDenial),
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BridgeHomeRetirementProgress {
    Advanced,
    Complete,
}

/// Same original source payload and position through interruption.
pub struct RetiringRuntimeBridge {
    sources: [HomeBridgeSource; 4],
    cursor: usize,
    _allocation: RetiredBridgeConfiguration,
}
pub struct RetiredRuntimeBridge {
    _custody: RetiringRuntimeBridge,
}
enum HomeBridgeSource {
    Erased(Arc<dyn Any + Send + Sync>),
    Source(RuntimeBridgeRelationalSource),
    Retired {
        _original: RetiredRuntimeBridgeRelationalSource,
    },
    LocalShareReleased,
}
struct RetiredBridgeConfiguration {
    _policy: BridgeRuntimePolicy,
    _diagnostics: BridgeDiagnosticsFacade,
    _diagnostic_sink: Arc<dyn DiagnosticSink>,
    _profile: Option<crate::input::envelope::BridgeAuthoritativeSourceProfile>,
    _sink: Arc<dyn InvalidationSink>,
    _sources: AdmittedSourceRegistry,
    _structural: AdmittedStructuralRegistry,
    _merge: AdmittedMergeRegistry,
    _mappings: FrozenMappingRegistry,
    _aspects: FrozenAspectMappingRegistry,
    _subscriptions: FrozenSubscriptionFamilyRegistry,
    _signal_key: u64,
    _signal: crate::source::BridgeSignalRuntimeCustody,
    _lowering: worth_signal::facade::SignalAspectLoweringOwner,
    _reservations: Arc<crate::execution_basis::BridgeExecutionBasisReservationRegistry>,
    _correspondences: crate::correspondence::SharedCorrespondenceAllocationRegistry,
    _dependencies: crate::correspondence::AdmittedSemanticDependencyRegistry,
}

impl RuntimeBridge {
    pub fn preflight_home_retirement(
        &self,
        native: &RelationalRuntime,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), BridgeHomeRetirementDenial> {
        request
            .consult()
            .map_err(BridgeHomeRetirementDenial::Execution)?;
        if self.snapshot_reader_pool.is_some()
            || self.continuity_lineage_source.is_some()
            || self.writeback_authority.is_some()
        {
            return Err(BridgeHomeRetirementDenial::Unsupported);
        }
        validate_source(self.committed_patch_source.as_ref(), native)?;
        validate_source(self.snapshot_read_source.as_ref(), native)?;
        if let Some(source) = self.truth_branch_head_source.as_ref() {
            validate_source(source.as_ref(), native)?;
        }
        if let Some(source) = self.source_adapter.as_ref() {
            validate_source(source.as_ref(), native)?;
        }
        Ok(())
    }

    /// Preflight precedes extraction; a refusal returns this original Bridge.
    /// All supplier payloads remain in the same retired allocation until reclaim.
    pub fn into_home_retirement(
        self,
        native: &RelationalRuntime,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RetiringRuntimeBridge, (Self, BridgeHomeRetirementDenial)> {
        if let Err(cause) = self.preflight_home_retirement(native, request) {
            return Err((self, cause));
        }
        let Self {
            policy,
            diagnostics,
            diagnostic_sink,
            committed_patch_source,
            authoritative_source_profile,
            snapshot_read_source,
            snapshot_reader_pool,
            signal_sink,
            truth_branch_head_source,
            continuity_lineage_source,
            writeback_authority,
            source_registry,
            source_adapter,
            structural_registry,
            merge_registry,
            mapping_registry,
            aspect_registry,
            subscription_family_registry,
            signal_runtime_key,
            signal_runtime_custody,
            signal_aspect_lowering_owner,
            execution_basis_reservations,
            correspondence_allocations,
            semantic_dependency_registry,
        } = self;
        // Preflight establishes these contain no unknown operational payload.
        drop((
            snapshot_reader_pool,
            continuity_lineage_source,
            writeback_authority,
        ));
        Ok(RetiringRuntimeBridge {
            sources: [
                HomeBridgeSource::Erased(committed_patch_source),
                HomeBridgeSource::Erased(snapshot_read_source),
                truth_branch_head_source
                    .map(|source| HomeBridgeSource::Erased(source))
                    .unwrap_or(HomeBridgeSource::LocalShareReleased),
                source_adapter
                    .map(|source| HomeBridgeSource::Erased(source))
                    .unwrap_or(HomeBridgeSource::LocalShareReleased),
            ],
            cursor: 0,
            _allocation: RetiredBridgeConfiguration {
                _policy: policy,
                _diagnostics: diagnostics,
                _diagnostic_sink: diagnostic_sink,
                _profile: authoritative_source_profile,
                _sink: signal_sink,
                _sources: source_registry,
                _structural: structural_registry,
                _merge: merge_registry,
                _mappings: mapping_registry,
                _aspects: aspect_registry,
                _subscriptions: subscription_family_registry,
                _signal_key: signal_runtime_key,
                _signal: signal_runtime_custody,
                _lowering: signal_aspect_lowering_owner,
                _reservations: execution_basis_reservations,
                _correspondences: correspondence_allocations,
                _dependencies: semantic_dependency_registry,
            },
        })
    }
}

impl RetiringRuntimeBridge {
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<BridgeHomeRetirementProgress, BridgeHomeRetirementDenial> {
        use BridgeHomeRetirementProgress as Progress;
        if self.cursor == self.sources.len() {
            return Ok(Progress::Complete);
        }
        request
            .consult()
            .map_err(BridgeHomeRetirementDenial::Execution)?;
        let slot = &mut self.sources[self.cursor];
        match slot {
            HomeBridgeSource::Retired { .. } | HomeBridgeSource::LocalShareReleased => {
                self.cursor += 1;
            }
            HomeBridgeSource::Erased(source) => {
                validate_source(source.as_ref(), native)?;
                let HomeBridgeSource::Erased(source) =
                    std::mem::replace(slot, HomeBridgeSource::LocalShareReleased)
                else {
                    unreachable!()
                };
                let source = source
                    .downcast::<RuntimeBridgeRelationalSource>()
                    .unwrap_or_else(|_| unreachable!("same checked original source"));
                if let Some(source) = Arc::into_inner(source) {
                    *slot = HomeBridgeSource::Source(source);
                }
            }
            HomeBridgeSource::Source(_) => {
                let HomeBridgeSource::Source(source) =
                    std::mem::replace(slot, HomeBridgeSource::LocalShareReleased)
                else {
                    unreachable!()
                };
                match source.try_retire_alias_for_request(request, native) {
                    Ok(retired) => *slot = HomeBridgeSource::Retired { _original: retired },
                    Err((source, cause)) => {
                        *slot = HomeBridgeSource::Source(source);
                        return Err(BridgeHomeRetirementDenial::NativeAlias(cause));
                    }
                }
            }
        }
        Ok(Progress::Advanced)
    }
    pub fn into_retired(self) -> Result<RetiredRuntimeBridge, Self> {
        if self.cursor == self.sources.len() {
            Ok(RetiredRuntimeBridge { _custody: self })
        } else {
            Err(self)
        }
    }
}
impl RetiredRuntimeBridge {
    pub fn reclaim(self) {
        drop(self);
    }
}

fn validate_source(
    value: &dyn Any,
    native: &RelationalRuntime,
) -> Result<(), BridgeHomeRetirementDenial> {
    value
        .downcast_ref::<RuntimeBridgeRelationalSource>()
        .ok_or(BridgeHomeRetirementDenial::Unsupported)?
        .preflight_alias_retirement(native)
        .map_err(BridgeHomeRetirementDenial::NativeAlias)
}
