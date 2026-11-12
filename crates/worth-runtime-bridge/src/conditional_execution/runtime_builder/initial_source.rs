//! Original Bridge installation follows Signal's retained initial publication.
use super::{authored_readmission::AuthoredSignalSource, BridgeConditionalRuntimeBuilder};
use crate::conditional_execution::{
    BridgeAuthoredInstallationBindings, BridgeAuthoredReadmissionDenial,
    BridgeAuthoredSectionDenial, BridgeReadmittedAuthoredAssembly,
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_signal::facade::durability::{
    SignalAuthoredInstallationRefused, SignalDurableMedium, SignalDurableTierVocabulary,
    SignalInitialSourceDenial, SignalInitialSourceProgress, SignalInitialSourcePublication,
    SignalInitialSourceStepRefusal, SignalPublishedInitialSource,
    SignalRecordedIdentityContribution, SignalRecordedSourcePoint,
};

#[derive(Debug)]
pub enum BridgeInitialSourceDenial {
    Signal(Box<SignalInitialSourceDenial>),
    Section(Box<BridgeAuthoredSectionDenial>),
    UnsupportedProgramFamily,
}
pub struct BridgeInitialSourcePreparationRefusal {
    builder: Box<BridgeConditionalRuntimeBuilder>,
    cause: BridgeInitialSourceDenial,
}
impl BridgeInitialSourcePreparationRefusal {
    pub fn cause(&self) -> &BridgeInitialSourceDenial {
        &self.cause
    }
    pub fn into_parts(
        self,
    ) -> (
        Box<BridgeConditionalRuntimeBuilder>,
        BridgeInitialSourceDenial,
    ) {
        (self.builder, self.cause)
    }
}
#[must_use]
pub struct BridgeInitialSourcePublication {
    builder: Box<BridgeConditionalRuntimeBuilder>,
    signal: SignalInitialSourcePublication<(), (), (), (), ()>,
    memory: Option<ExecutionMemoryReservation>,
}
#[must_use]
pub struct BridgePublishedInitialSource {
    builder: Box<BridgeConditionalRuntimeBuilder>,
    signal: SignalPublishedInitialSource<(), (), (), (), ()>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl BridgeConditionalRuntimeBuilder {
    /// Prepare initial authored bytes from this actual installed program. The
    /// returned original must remain Home-owned until its serving handoff.
    pub fn prepare_initial_authored_source(
        mut self: Box<Self>,
        medium: Arc<dyn SignalDurableMedium>,
        contribution: SignalRecordedIdentityContribution,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeInitialSourcePublication, BridgeInitialSourcePreparationRefusal> {
        if !self.managed_clocks.is_empty() || self.runtime.owned_async_declaration_count() != 0 {
            return Err(BridgeInitialSourcePreparationRefusal {
                builder: self,
                cause: BridgeInitialSourceDenial::UnsupportedProgramFamily,
            });
        }
        let memory = match execution
            .reserve_retained_memory(std::mem::size_of::<BridgeInitialSourcePublication>() as u64)
        {
            Ok(memory) => memory,
            Err(cause) => {
                return Err(BridgeInitialSourcePreparationRefusal {
                    builder: self,
                    cause: BridgeInitialSourceDenial::Section(Box::new(
                        BridgeAuthoredSectionDenial::Request(cause),
                    )),
                })
            }
        };
        let vocabulary = match SignalDurableTierVocabulary::declare([((), "bridge")]) {
            Ok(vocabulary) => vocabulary,
            Err(_) => {
                return Err(BridgeInitialSourcePreparationRefusal {
                    builder: self,
                    cause: BridgeInitialSourceDenial::Section(Box::new(
                        BridgeAuthoredSectionDenial::Malformed,
                    )),
                })
            }
        };
        let program = self.take_signal();
        let mut section_refusal = None;
        let prepared = program.prepare_initial_authored_source(
            medium,
            contribution,
            vocabulary,
            execution,
            |context| {
                self.capture_initial_section(context, execution)
                    .map_err(|cause| {
                        section_refusal = Some(cause);
                        SignalAuthoredInstallationRefused
                    })
            },
        );
        match prepared {
            Ok(signal) => Ok(BridgeInitialSourcePublication {
                builder: self,
                signal,
                memory,
            }),
            Err(refusal) => {
                let (program, cause) = refusal.into_parts();
                *self
                    .runtime
                    .signal_runtime
                    .get_mut()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(*program);
                let cause = match section_refusal {
                    Some(cause) => BridgeInitialSourceDenial::Section(Box::new(cause)),
                    None => BridgeInitialSourceDenial::Signal(Box::new(cause)),
                };
                Err(BridgeInitialSourcePreparationRefusal {
                    builder: self,
                    cause,
                })
            }
        }
    }
}
impl BridgeInitialSourcePublication {
    pub fn advance(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalInitialSourceProgress, SignalInitialSourceStepRefusal> {
        self.signal.advance(execution)
    }
    pub fn definition(
        &self,
    ) -> &worth_signal::facade::durability::SignalDefinitionTransitionArtifact {
        self.signal.definition()
    }
    pub fn advance_artifact(
        &self,
    ) -> &worth_signal::facade::durability::SignalBranchAdvanceArtifact {
        self.signal.advance_artifact()
    }
    pub fn try_into_published(self) -> Result<BridgePublishedInitialSource, Self> {
        match self.signal.try_into_published() {
            Ok(signal) => Ok(BridgePublishedInitialSource {
                builder: self.builder,
                signal,
                _memory: self.memory,
            }),
            Err(signal) => Err(Self {
                builder: self.builder,
                signal,
                memory: self.memory,
            }),
        }
    }
}
impl BridgePublishedInitialSource {
    pub fn source_point(&self) -> SignalRecordedSourcePoint {
        self.signal.source_point()
    }
    pub fn readmit_authored_owner(
        self,
        bindings: &mut dyn BridgeAuthoredInstallationBindings,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<BridgeReadmittedAuthoredAssembly>, BridgeAuthoredReadmissionDenial> {
        self.builder.readmit_source(
            AuthoredSignalSource::Initial(self.signal),
            bindings,
            execution,
        )
    }
}
