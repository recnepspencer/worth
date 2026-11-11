use super::*;

/// Composing runtime's original prepaid coordination, retained with World's
/// existing creation row through caller loss. It grants no owner-effect authority.
pub trait RuntimeWorldBranchCreationCompletion: std::fmt::Debug + Send + Sync {
    fn completed(
        &mut self,
        source: &ProductBranchObservation,
        destination: &ProductBranchObservation,
    );
}

/// The actual World release ACK terminated the original source-cell resource.
#[derive(Debug)]
#[must_use = "the receipt names the original that was durably released"]
pub struct RuntimeWorldBranchCreationRelease {
    original: crate::branch::ActualWorldCreationRelease,
}
impl RuntimeWorldBranchCreationRelease {
    pub(crate) fn from_original(
        original: worth_proof::DurablyCommitted<crate::branch::ActualWorldCreationRelease>,
    ) -> Self {
        original.consume(|original| Self { original })
    }
    pub fn association(&self) -> &worth_foundational::facade::CompositeAttemptAssociation {
        self.original.association()
    }
}

/// The attached medium proved this exact original had no World durable effect.
#[derive(Debug)]
pub struct RuntimeWorldBranchCreationNoEffect {
    original:
        worth_proof::ProvenNoEffectPublication<crate::durability::ActualWorldNoEffectPublication>,
    cause: crate::durability::WorldDurableMediumDenial,
}
impl RuntimeWorldBranchCreationNoEffect {
    pub(crate) fn from_original(refusal: crate::durability::WorldPublicationRefusal) -> Self {
        let (original, cause) = refusal.into_parts();
        Self { original, cause }
    }
    pub fn cause(&self) -> &crate::durability::WorldDurableMediumDenial {
        &self.cause
    }
    pub fn association(&self) -> &worth_foundational::facade::CompositeAttemptAssociation {
        let crate::durability::WorldRecord::CreationIntent(intent) =
            self.original.original().subject().record()
        else {
            unreachable!("original creation")
        };
        &intent.association
    }
}

/// A Home creation carries the programme actually observed for its selected
/// branch. The Home descriptor's global revision cannot fill this proof.
#[derive(Debug)]
pub struct RuntimeWorldHomeBranchCreationRequest {
    pub(crate) source: ProductBranchObservation,
    pub(crate) program_source: worth_proof::AdmittedBranchProgramSource,
    pub(crate) intent: ProductBranchCreationIntent,
    pub(crate) completion: Option<Box<dyn RuntimeWorldBranchCreationCompletion>>,
}
impl RuntimeWorldHomeBranchCreationRequest {
    pub fn with_completion(
        mut self,
        completion: Box<dyn RuntimeWorldBranchCreationCompletion>,
    ) -> Self {
        self.completion = Some(completion);
        self
    }
    pub fn new(
        source: ProductBranchObservation,
        program_source: worth_proof::AdmittedBranchProgramSource,
        intent: ProductBranchCreationIntent,
    ) -> Self {
        Self {
            source,
            program_source,
            intent,
            completion: None,
        }
    }
}

/// The memory and Home profiles have different source authority. Neither can
/// silently substitute for the other, and retained-memory roots stay explicit.
pub(crate) enum RuntimeWorldBranchCreationRequest<'a, 'request, 'authority> {
    Memory {
        execution: &'a worth_execution::ExecutionRequest<'request, 'authority>,
        source: ProductBranchObservation,
        intent: ProductBranchCreationIntent,
        cancellation: &'a RuntimeWorldCancellationToken,
        retained: Option<worth_relational::facade::branch::AdmittedRelationalRetainedForkSource>,
    },
    Home {
        execution: &'a worth_execution::ExecutionRequest<'request, 'authority>,
        request: RuntimeWorldHomeBranchCreationRequest,
        cancellation: &'a RuntimeWorldCancellationToken,
    },
}
impl<'a, 'request, 'authority> RuntimeWorldBranchCreationRequest<'a, 'request, 'authority> {
    pub(crate) fn new(
        execution: &'a worth_execution::ExecutionRequest<'request, 'authority>,
        source: ProductBranchObservation,
        intent: ProductBranchCreationIntent,
        cancellation: &'a RuntimeWorldCancellationToken,
    ) -> Self {
        Self::Memory {
            execution,
            source,
            intent,
            cancellation,
            retained: None,
        }
    }
    pub(crate) fn home(
        execution: &'a worth_execution::ExecutionRequest<'request, 'authority>,
        request: RuntimeWorldHomeBranchCreationRequest,
        cancellation: &'a RuntimeWorldCancellationToken,
    ) -> Self {
        Self::Home {
            execution,
            request,
            cancellation,
        }
    }
    pub(crate) fn retained(
        execution: &'a worth_execution::ExecutionRequest<'request, 'authority>,
        source: crate::branch::RetainedProductBranchCreationSource,
        intent: ProductBranchCreationIntent,
        cancellation: &'a RuntimeWorldCancellationToken,
    ) -> Self {
        Self::Memory {
            execution,
            source: source.observation,
            intent,
            cancellation,
            retained: Some(source.native),
        }
    }
}
