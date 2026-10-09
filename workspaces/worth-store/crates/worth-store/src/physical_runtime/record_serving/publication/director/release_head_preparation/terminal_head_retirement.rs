//! Prepares the record-less WAL member for one terminal head retired. The
//! sealed authority names the fenced root, the attested head and the bound
//! SessionDeclared record; this module plans only the one-key tree removal.

use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    PersistedReleaseHeadClaim, PersistedTerminalReleaseHeadRetirementV1, ReleaseCustodyHeadEntryV1,
    ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadTransitionV1, SelectedRecordContentClass,
};

use super::super::super::durable_preparation::CanonicalPayloadMaterializationObservation;
use super::super::durable_preparation::{map_record_denial, PhysicalMutationPreparationAdmission};
use super::super::selected_segment_rewrite::{admitted_terminal, record_identity_bytes};
use super::super::RecordPublicationDirector;
use super::{damaged, head_denial, release_head_limits, release_head_reservation_bytes};
use crate::physical_runtime::durability::{
    PhysicalMutationOperationFamily, PreparedPhysicalDataPlan, TerminalHeadRetirementAuthority,
};
use crate::physical_runtime::record_serving::{
    access::release_custody_head::read_release_head_path, AdmittedRecordPlacementPolicy,
    PreparedPhysicalRootProjection, RecordAppendBatch, RecordAppendDenial, RecordAppendError,
};
use crate::physical_runtime::{
    PhysicalManifestCapacityTransition, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalMutationResourceShape,
    PhysicalMutationRuntimeOwner, PreparedPhysicalMutation, PreparedPhysicalMutationContext,
};

const REQUEST_DOMAIN: &[u8] = b"store.physical.terminal-head-retired.request.v1";

impl RecordPublicationDirector {
    /// Every fallible planning step precedes idempotency admission, so a
    /// denial leaves no binding behind.
    pub(in crate::physical_runtime) fn prepare_terminal_head_retirement(
        &self,
        authority: &TerminalHeadRetirementAuthority<'_>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        if let Err(outcome) = self.require_preparation_health() {
            return outcome;
        }
        if !placement.admits(self.format) {
            return map_record_denial(RecordAppendDenial::PlacementFormatMismatch);
        }
        let (root, digest) = match self.plan_terminal_head_retirement(authority, placement) {
            Ok(planned) => planned,
            Err(RecordAppendError::Denied(denial)) => return map_record_denial(denial),
            Err(_) => return map_record_denial(RecordAppendDenial::PublishedLayoutDamaged),
        };
        let group_queue_admission = match self.group_queue_admission_tick() {
            Ok(tick) => tick,
            Err(outcome) => return outcome,
        };
        let admitted = match self.admit_mutation_preparation(
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            digest,
            request,
            PhysicalMutationOperationFamily::TerminalHeadRetirement,
        ) {
            Ok(admitted) => admitted,
            Err(outcome) => return outcome,
        };
        let PhysicalMutationPreparationAdmission::Prepared(admitted) = admitted else {
            return admitted_terminal(admitted);
        };
        let prepared = PreparedPhysicalMutation::new(
            admitted.admission,
            RecordAppendBatch::from_prepared_record_bytes(Vec::new()),
            CanonicalPayloadMaterializationObservation::default(),
            PreparedPhysicalMutationContext {
                blob_record_kind: None,
                selected_content_class: SelectedRecordContentClass::UnknownLegacy,
                inline_only: false,
                derived_directory_basis: None,
                reuse_declaration_basis: None,
                placement,
                manifest_capacity_transition: PhysicalManifestCapacityTransition::PreserveCurrent,
                deadline: admitted.deadline,
                group_queue_admission,
                signal_profile: self.signal_profile,
                durability_policy_basis: self.durability_policy_basis.clone(),
                resources: PhysicalMutationResourceShape::prepared(0, 0),
                start: PhysicalMutationRuntimeOwner::start_port(&self.mutations),
                selected_segment_rewrite: false,
                rewrite_pages: 0,
                source_root_generation: 0,
                rewrite_anchor: None,
            },
        )
        .attach_plans(PreparedPhysicalDataPlan::TerminalHeadRetirement, root);
        if !authority
            .attempt()
            .register_terminal_head_retirement(prepared.mutation_identity())
        {
            let _ = self.cancel_prepared_before_group_seal(prepared);
            return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable);
        }
        TransitionOutcome::success(PhysicalMutationPreparationSuccess::Prepared(prepared)).into()
    }

    /// Plans the one-key removal against the fenced root and seals it as the
    /// member's whole root projection.
    fn plan_terminal_head_retirement(
        &self,
        authority: &TerminalHeadRetirementAuthority<'_>,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Result<(PreparedPhysicalRootProjection, [u8; 32]), RecordAppendError> {
        let (root, _) = self.root_owner.snapshot();
        if root.root_cell() != authority.root() {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::ReclaimFenceUnavailable,
            ));
        }
        let format = self.format.declaration();
        let reservation = release_head_reservation_bytes(format)
            .and_then(NonZeroU64::new)
            .ok_or_else(damaged)?;
        let allocation = self
            .residency
            .begin_foreground_write_operation(reservation)
            .map_err(|denial| {
                RecordAppendError::Denied(RecordAppendDenial::from_residency(denial))
            })?;
        let limits = release_head_limits(format)?;
        let source_path = read_release_head_path(
            &allocation,
            self.residency.clone(),
            self.format,
            self.access,
            &root,
            authority.key(),
            limits,
        )?;
        let transition = ReleaseCustodyHeadTransitionV1::plan(
            root.release_custody_head_root(),
            root.next_release_custody_head_block(),
            &source_path,
            ReleaseCustodyHeadMutationV1::RetireTerminal {
                expected_prior: authority.expected_prior(),
            },
            root.generation().checked_add(1).ok_or_else(damaged)?,
            root.tree_identity(),
            format,
            limits,
        )
        .map_err(head_denial)?;
        let retirement = PersistedTerminalReleaseHeadRetirementV1::new(
            root.generation(),
            authority.declaration_record(),
            authority.declaration_frame_sha256(),
            root.tree_identity(),
            authority.source_basis(),
            source_path,
            transition,
            format,
        )
        .map_err(|_| damaged())?;
        let digest = request_digest(&retirement);
        // The successor restamps one free-space header frame; the claim set
        // below adds its own framed head-tree bytes.
        let header_frame =
            NonZeroU64::new(u64::from(format.page_size().bytes())).ok_or_else(damaged)?;
        let mut projection =
            PreparedPhysicalRootProjection::record_less(root, placement, header_frame);
        projection
            .set_release_head_claim(PersistedReleaseHeadClaim::TerminalHeadRetired(retirement))
            .ok_or(RecordAppendError::Denied(
                RecordAppendDenial::PhysicalPressure,
            ))?;
        Ok((projection, digest))
    }
}

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn admit_terminal_head_retirement(
        &self,
        claim: &mut crate::physical_runtime::durability::PhysicalBlobSessionClaim,
        inspector: crate::physical_runtime::PhysicalProtectedRootObservation,
        key: worth_store_physical_format::ReleaseCustodyHeadKeyV1,
        charge: crate::physical_runtime::durability::ReleaseHeadCapacityCharge,
    ) -> Result<
        crate::physical_runtime::durability::AdmittedTerminalHeadRetirement,
        crate::physical_runtime::durability::TerminalHeadRetirementAdmissionDenial,
    > {
        self.root_owner
            .admit_terminal_head_retirement(claim, inspector, key, charge)
    }

    pub(in crate::physical_runtime) fn attest_no_terminal_head_retry_claim(
        &self,
        publication: &crate::physical_runtime::durability::TerminalHeadPublicationExcluded,
    ) -> Result<
        crate::physical_runtime::durability::TerminalHeadNoRetryClaim,
        crate::physical_runtime::durability::TerminalHeadRetryClaimDenial,
    > {
        self.idempotency
            .attest_no_terminal_head_retry_claim(publication)
    }

    pub(in crate::physical_runtime) fn commit_terminal_head_retirement(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
        completed: &crate::physical_runtime::CompletedPhysicalMutation,
    ) -> Result<(), crate::physical_runtime::durability::ReleaseCertificateCapacityDenial> {
        self.root_owner
            .commit_terminal_head_retirement(attempt, completed)
    }
}

/// The idempotency fingerprint names the exact head removed and the
/// SessionDeclared record the retirement binds.
fn request_digest(retirement: &PersistedTerminalReleaseHeadRetirementV1) -> [u8; 32] {
    let mut entry = [0_u8; ReleaseCustodyHeadEntryV1::ENCODED_BYTES];
    retirement.expected_prior().encode_into(&mut entry);
    let mut digest = Sha256::new();
    digest.update(REQUEST_DOMAIN);
    digest.update(entry);
    digest.update(record_identity_bytes(retirement.declaration_record()));
    digest.update(retirement.declaration_frame_sha256());
    digest.finalize().into()
}
