//! Immutable successor execution meaning, prepared on the actual Native owner.
use crate::branch::{AdmittedRelationalBranchBasis, RelationalBranchRootSchemaAuthority};
use crate::runtime::{RelationalRuntime, SchemaContractRuntimeSubsystem};
use crate::schema::data::RelationalSchemaRegistry;
use crate::validation::candidate_source::ManagedIndexSourceProgram;
use crate::validation::{data::CustomInvariantRegistration, FrozenCustomInvariantRegistry};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};
use worth_foundational::facade::HomeProgramTransitionCatalogDigest;

mod body_digest;
mod body_validation;
pub use body_digest::{home_program_transition_body_digest, RelationalHomeProgramBodyDenial};
mod catalog_digest;
mod predecessor_catalog;
mod retention;
pub(crate) use predecessor_catalog::RelationalHomeProgramPredecessorCatalog;
mod catalog_receipt;
pub use catalog_receipt::RelationalHomeProgramCatalogReceipt;
mod catalog_source;
pub use catalog_source::RelationalHomeProgramCatalogReadmissionSource;
mod coverage;
mod schema_compilation_denial;
mod step_admission;
mod target_step;
pub(crate) use coverage::RelationalHomeTargetCoverage;
pub(crate) use step_admission::AdmittedRelationalHomeProgramStep;
pub use step_admission::RelationalHomeProgramStepContracts;
pub(crate) use target_step::AdmittedRelationalHomeProgramTarget;

#[derive(Debug)]
pub enum RelationalHomeProgramContractDenial {
    Request(WorkCeilingDenial),
    ForeignRuntime,
    Source(crate::derived_relation::DerivedRelationDenial),
    DuplicateInvariant(crate::validation::DuplicateCustomInvariantRegistration),
    SchemaCapture,
    SchemaImage(crate::durability::data::RelationalRecoveryStop),
    Execution(crate::execution::RelationalExecutionDenialCause),
    SchemaAuthorityExhausted,
    GenerationExhausted,
    CapacityOverflow,
    CatalogEncoding,
    UnboundHome,
    Admission(worth_proof::HomeProgramTransitionAdmissionDenial),
    Plan(worth_foundational::facade::HomeProgramTransitionDescriptionDenial),
    TargetCatalogMismatch,
    IndexDefinitions(crate::runtime::RelationalIndexDefinitionPreparationDenial),
    TargetStepMismatch,
    CallerEnvelope(worth_foundational::facade::HomeProgramTransitionCallerDenial),
    SourceContractChanged,
    SourceBasisMismatch,
    Transaction(crate::mvcc::RelationalBranchTransactionAdmissionDenial),
    SchemaTransition(crate::runtime::RelationalSchemaTransitionAdmissionDenial),
}

/// Prepared compiled meaning grants no transaction, publication or Source read.
#[derive(Debug)]
pub struct PreparedRelationalHomeProgramContracts {
    pub(crate) meaning: Arc<RelationalRootExecutionContracts>,
    pub(crate) receiver_epoch: crate::runtime::RelationalRuntimeConfigurationSnapshot,
    pub(crate) index_definitions: Option<Arc<crate::runtime::PreparedRelationalIndexDefinitions>>,
}
impl PreparedRelationalHomeProgramContracts {
    /// Attach privately prepared definitions from this actual Native owner.
    pub fn with_index_definitions(
        mut self,
        definitions: crate::runtime::PreparedRelationalIndexDefinitions,
    ) -> Result<Self, RelationalHomeProgramContractDenial> {
        if definitions.runtime != self.meaning.runtime {
            return Err(RelationalHomeProgramContractDenial::ForeignRuntime);
        }
        self.index_definitions = Some(Arc::new(definitions));
        Ok(self)
    }

    pub fn catalog_digest(&self) -> HomeProgramTransitionCatalogDigest {
        self.meaning.digest
    }
    pub fn schema_digest(&self) -> worth_foundational::facade::HomeProgramTransitionSchemaDigest {
        worth_foundational::facade::HomeProgramTransitionSchemaDigest::from_recorded(
            self.meaning.schema.authority_digest(),
        )
    }

    /// Describe the actual prepared schema and its full recorded carrier.
    /// Neither this value nor its digests grants an executable Home step.
    pub fn target_meaning(
        &self,
        request: &ExecutionRequest<'_, '_>,
        step: u32,
    ) -> Result<
        worth_foundational::facade::HomeProgramTransitionTargetMeaning,
        RelationalHomeProgramContractDenial,
    > {
        use worth_foundational::facade::{
            HomeProgramTransitionSchemaCarrierDigest, HomeProgramTransitionTargetMeaning,
        };
        let image = self
            .meaning
            .schema
            .recorded_image(request)
            .map_err(RelationalHomeProgramContractDenial::SchemaImage)?;
        Ok(HomeProgramTransitionTargetMeaning::from_recorded(
            step,
            self.catalog_digest(),
            self.schema_digest(),
            HomeProgramTransitionSchemaCarrierDigest::from_recorded(image.carrier_digest()),
        ))
    }
}

/// A root retains its own admitted executors. It never mutates the receiver's
/// construction configuration or borrows a disposable author's request.
#[derive(Debug)]
pub(crate) struct RelationalRootExecutionContracts {
    identity: Arc<catalog_receipt::RelationalHomeProgramCatalogIdentity>,
    pub(crate) runtime: u64,
    pub(crate) schema: Arc<RelationalBranchRootSchemaAuthority>,
    pub(crate) contracts: Arc<SchemaContractRuntimeSubsystem>,
    pub(crate) digest: HomeProgramTransitionCatalogDigest,
    _memory: Option<ExecutionMemoryReservation>,
}

impl RelationalRuntime {
    /// Release the before-service writer fence only with the Home owner's
    /// actual final binding acknowledgement, checked against its attachment.
    pub fn complete_home_program_transition(
        &self,
        completed: worth_proof::HomeProgramTransitionBindingCommitted,
    ) -> Result<(), crate::durability::medium::RelationalHomeProgramWriteDenial> {
        self.durability
            .medium()
            .ok_or(
                crate::durability::medium::RelationalHomeProgramWriteDenial::TransitionNotRequested,
            )?
            .complete_home_program_transition(completed)
    }

    /// Compile the complete successor without installing it or invoking a
    /// provider. Only a committed Home step can later admit it for execution.
    pub fn prepare_home_program_contracts(
        &self,
        request: &ExecutionRequest<'_, '_>,
        predecessor: &AdmittedRelationalBranchBasis,
        registry: RelationalSchemaRegistry,
        invariants: Vec<CustomInvariantRegistration>,
        sources: Vec<ManagedIndexSourceProgram>,
    ) -> Result<PreparedRelationalHomeProgramContracts, RelationalHomeProgramContractDenial> {
        use RelationalHomeProgramContractDenial as Denial;
        request.consult().map_err(Denial::Request)?;
        if predecessor.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(Denial::ForeignRuntime);
        }
        let snapshot = self.configuration_binding().snapshot();
        let source_generation = predecessor.inner.root.execution_contracts.get().map_or(
            snapshot.schema_contract_runtime.custom_invariant_generation,
            |source| source.contracts.custom_invariant_generation,
        );
        let generation = source_generation
            .checked_add(1)
            .ok_or(Denial::GenerationExhausted)?;
        let schema = RelationalBranchRootSchemaAuthority::prepare_home_target(
            request,
            self.history.fork_binding().root_identity_issuer(),
            registry,
            snapshot
                .config
                .schema
                .descriptor_semantics_policy
                .current_write_version(),
            predecessor.inner.root.schema_authority(),
        )
        .map_err(schema_compilation_denial::translate)?;
        let digest =
            catalog_digest::digest(schema.authority_digest(), &invariants, &sources, request)?;
        let custom = FrozenCustomInvariantRegistry::compile(
            schema.registry(),
            invariants,
            sources,
            crate::schema::SchemaCompilationControl::Request(*request),
        )
        .map_err(|cause| match cause {
            crate::validation::CustomInvariantCompilationDenial::Source(cause) => {
                Denial::Source(cause)
            }
            crate::validation::CustomInvariantCompilationDenial::Duplicate(cause) => {
                Denial::DuplicateInvariant(cause)
            }
            crate::validation::CustomInvariantCompilationDenial::Execution(cause) => {
                Denial::Execution(cause)
            }
            crate::validation::CustomInvariantCompilationDenial::CapacityOverflow => {
                Denial::CapacityOverflow
            }
        })?;
        let contracts = SchemaContractRuntimeSubsystem::for_schema_compilation(
            schema.registry(),
            schema.aspect_plans_arc(),
            custom,
            generation,
            true,
            crate::schema::SchemaCompilationControl::Request(*request),
        )
        .map_err(Denial::Execution)?;
        self.retain_home_program_contracts(request, schema, Arc::new(contracts), digest, snapshot)
    }
}
