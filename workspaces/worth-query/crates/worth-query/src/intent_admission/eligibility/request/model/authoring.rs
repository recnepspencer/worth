use crate::basis_lifecycle::{normalize_raw_basis_intent, RawBasisIntent};
use crate::identity::hash_parts;
use crate::projection_consumption::ProjectionConsumptionDeclaration;
use crate::runtime::WorthQueryIntentDeclaration;

use super::super::super::seeds::{
    WorthQueryAuthoritativeMutationBatchIntentSeed, WorthQueryAuthoritativeMutationIntentSeed,
    WorthQueryDerivedViewIntentSeed, WorthQueryExistingTruthProbeIntentSeed,
    WorthQueryGenericInspectionIntentSeed, WorthQueryLiveReadIntentSeed,
    WorthQueryReadExecutionIntentSeed,
};
use crate::intent_admission::{
    intent_family_for_entrypoint, WorthQueryIntentAdmissionCoveredEntrypoint,
    WorthQueryIntentAdmissionFamily, WorthQueryIntentViolationDecision,
};

use super::{WorthQueryIntentAdmissionRequestPayload, WorthQueryRawIntentAdmissionRequest};

impl WorthQueryRawIntentAdmissionRequest {
    pub(crate) fn authoritative_runtime_entrypoint(
        declaration: WorthQueryIntentDeclaration,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteIntent,
            declaration.name().to_string(),
            declaration.input_digest().to_string(),
            WorthQueryIntentAdmissionRequestPayload::RuntimeIntent(declaration),
        )
    }

    pub(crate) fn effect_runtime_entrypoint(
        declaration: WorthQueryIntentDeclaration,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteNextEffectWriteIntent,
            declaration.name().to_string(),
            declaration.input_digest().to_string(),
            WorthQueryIntentAdmissionRequestPayload::RuntimeIntent(declaration),
        )
    }

    pub(crate) fn authoritative_write_entrypoint(
        seed: WorthQueryAuthoritativeMutationIntentSeed,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        let intent_name = seed.command_label();
        let input_digest = seed.command_input_digest().map_err(|cause| {
            WorthQueryIntentViolationDecision::identity_refused(
                intent_family_for_entrypoint(
                    WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteScalarWrite,
                ),
                WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteScalarWrite,
                cause,
            )
        })?;
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteScalarWrite,
            intent_name,
            input_digest,
            WorthQueryIntentAdmissionRequestPayload::AuthoritativeMutation(seed),
        )
    }

    pub(crate) fn authoritative_write_batch_entrypoint(
        seed: WorthQueryAuthoritativeMutationBatchIntentSeed,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        let intent_name = seed.batch_label();
        let input_digest = seed.batch_input_digest().map_err(|cause| {
            WorthQueryIntentViolationDecision::identity_refused(
                intent_family_for_entrypoint(
                    WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteBatchWrite,
                ),
                WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteBatchWrite,
                cause,
            )
        })?;
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteBatchWrite,
            intent_name,
            input_digest,
            WorthQueryIntentAdmissionRequestPayload::AuthoritativeMutationBatch(seed),
        )
    }

    pub(crate) fn basis_observation_lane(
        raw: RawBasisIntent,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        let normalized = normalize_raw_basis_intent(raw, "observation").map_err(|denial| {
            WorthQueryIntentViolationDecision::new(
                WorthQueryIntentAdmissionFamily::BasisUseIntent,
                WorthQueryIntentAdmissionCoveredEntrypoint::BasisObservation,
                "raw-basis-intent",
                denial.message(),
                "basis-observation-raw-intent-rejected",
                "basis-observation-raw-intent-rejected",
            )
        })?;
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::BasisObservation,
            format!("basis.observation.{}", normalized.family().as_str()),
            normalized.normalized_digest().to_string(),
            WorthQueryIntentAdmissionRequestPayload::BasisObservation(normalized),
        )
    }

    pub(crate) fn read_family_entrypoint(
        seed: WorthQueryReadExecutionIntentSeed,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteReadFamily,
            seed.request_label().to_string(),
            seed.request_input_digest().to_string(),
            WorthQueryIntentAdmissionRequestPayload::ReadExecution(seed),
        )
    }

    pub(crate) fn read_family_in_basis_context_entrypoint(
        seed: WorthQueryReadExecutionIntentSeed,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteReadFamilyInBasisContext,
            seed.request_label().to_string(),
            seed.request_input_digest().to_string(),
            WorthQueryIntentAdmissionRequestPayload::ReadExecution(seed),
        )
    }

    pub(crate) fn live_read_entrypoint(
        seed: WorthQueryLiveReadIntentSeed,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteLiveRead,
            seed.request_label().to_string(),
            seed.request_input_digest().to_string(),
            WorthQueryIntentAdmissionRequestPayload::LiveReadExecution(seed),
        )
    }

    pub(crate) fn derived_materialization_entrypoint(
        seed: WorthQueryDerivedViewIntentSeed,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        let intent_name = seed.request_label("materialize");
        let input_digest = seed.request_input_digest("materialize");
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedMaterialization,
            intent_name,
            input_digest,
            WorthQueryIntentAdmissionRequestPayload::DerivedViewExecution(seed),
        )
    }

    pub(crate) fn derived_inspection_entrypoint(
        seed: WorthQueryDerivedViewIntentSeed,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        let intent_name = seed.request_label("inspect");
        let input_digest = seed.request_input_digest("inspect");
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedInspection,
            intent_name,
            input_digest,
            WorthQueryIntentAdmissionRequestPayload::DerivedViewExecution(seed),
        )
    }

    pub(crate) fn generic_inspection_entrypoint(
        seed: WorthQueryGenericInspectionIntentSeed,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteUnifiedInspection,
            seed.request_label().as_str().to_string(),
            seed.request_input_digest().to_string(),
            WorthQueryIntentAdmissionRequestPayload::GenericInspection(seed),
        )
    }

    pub(crate) fn existing_truth_probe_entrypoint(
        seed: WorthQueryExistingTruthProbeIntentSeed,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteExistingTruthProbeRouting,
            seed.request_label(),
            seed.request_input_digest(),
            WorthQueryIntentAdmissionRequestPayload::ExistingTruthProbeRouting(seed),
        )
    }

    pub(crate) fn projection_consumption(
        declaration: ProjectionConsumptionDeclaration,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        let source_family = declaration.source().family().as_str().to_string();
        let declaration_digest = declaration.declaration_digest().to_string();
        Self::new(
            WorthQueryIntentAdmissionCoveredEntrypoint::ProjectionConsumption,
            format!("projection.consume.{source_family}"),
            declaration_digest,
            WorthQueryIntentAdmissionRequestPayload::ProjectionConsumption(declaration),
        )
    }

    pub(crate) fn deferred_neighbor(
        entrypoint: WorthQueryIntentAdmissionCoveredEntrypoint,
        declaration: WorthQueryIntentDeclaration,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        Self::new(
            entrypoint,
            declaration.name().to_string(),
            declaration.input_digest().to_string(),
            WorthQueryIntentAdmissionRequestPayload::RuntimeIntent(declaration),
        )
    }

    fn new(
        entrypoint: WorthQueryIntentAdmissionCoveredEntrypoint,
        intent_name: String,
        input_digest: String,
        payload: WorthQueryIntentAdmissionRequestPayload,
    ) -> Result<Self, WorthQueryIntentViolationDecision> {
        let family = intent_family_for_entrypoint(entrypoint);
        let source_label = match &payload {
            WorthQueryIntentAdmissionRequestPayload::RuntimeIntent(declaration) => {
                declaration.source_lane().as_str().to_string()
            }
            WorthQueryIntentAdmissionRequestPayload::AuthoritativeMutation(seed) => {
                format!(
                    "authoritative-mutation:{}",
                    seed.command().mutation_family().as_str()
                )
            }
            WorthQueryIntentAdmissionRequestPayload::AuthoritativeMutationBatch(seed) => {
                format!("authoritative-mutation-batch:{}", seed.commands().len())
            }
            WorthQueryIntentAdmissionRequestPayload::ReadExecution(seed) => format!(
                "read-execution:{}:{}",
                seed.read_family().family_digest(),
                seed.basis_context()
                    .map(|context| context.family().as_str())
                    .unwrap_or("runtime-current")
            ),
            WorthQueryIntentAdmissionRequestPayload::LiveReadExecution(seed) => {
                format!("live-read-execution:{}", seed.live_view_digest())
            }
            WorthQueryIntentAdmissionRequestPayload::DerivedViewExecution(seed) => {
                format!("derived-view-execution:{}", seed.view_name())
            }
            WorthQueryIntentAdmissionRequestPayload::GenericInspection(seed) => {
                format!("generic-inspection:{}", seed.request_input_digest())
            }
            WorthQueryIntentAdmissionRequestPayload::ExistingTruthProbeRouting(seed) => {
                format!(
                    "existing-truth-probe-routing:{}",
                    seed.request().binding().binding_digest()
                )
            }
            WorthQueryIntentAdmissionRequestPayload::BasisObservation(_) => {
                "basis-observation".to_string()
            }
            WorthQueryIntentAdmissionRequestPayload::ProjectionConsumption(_) => {
                "projection-consumption".to_string()
            }
        };
        let request_digest = hash_parts(&[
            "worth_query_raw_intent_admission_request_v1".to_string(),
            format!("family:{}", family.as_str()),
            format!("entrypoint:{}", entrypoint.as_str()),
            format!("intent:{intent_name}"),
            format!("input:{input_digest}"),
            format!("source:{source_label}"),
        ]);
        Ok(Self {
            family,
            entrypoint,
            intent_name,
            input_digest,
            payload,
            request_digest,
        })
    }
}
