use std::sync::Arc;
use worth_proof::ProofOutcome;
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::{
    FilesystemBackendProfile, FilesystemMediaOwner, FilesystemMediaOwnerAdmissionDenial,
    FilesystemQualificationMode, FilesystemQualificationRequest, MediaQualificationDeferred,
    MediaQualificationDenial, MediaQualificationFailure, MediaQualificationIdentity,
    MediaQualificationPostOwnershipCause, MediaQualificationRebindRequired,
    MediaQualificationStale, QualifiedMediaCapabilities, RootProfileQualificationBasis,
    RootProfileQualificationReport,
};

pub type AdmittedFilesystemMedia = ProofOutcome<
    QualifiedFilesystemMedia,
    MediaQualificationDenial,
    MediaQualificationDeferred,
    MediaQualificationStale,
    MediaQualificationRebindRequired,
    MediaQualificationFailure,
>;

pub struct QualifiedFilesystemMedia {
    owner: FilesystemMediaOwner,
    profile: FilesystemBackendProfile,
    basis: RootProfileQualificationBasis,
    capabilities: QualifiedMediaCapabilities,
    execution_capability: crate::AdmittedBackendCapabilityWitness,
    store_identity: StableStoreIdentity,
    mode: FilesystemQualificationMode,
}

impl QualifiedFilesystemMedia {
    pub(super) const fn artifact_tree_owner(&self) -> &FilesystemMediaOwner {
        &self.owner
    }

    pub fn profile(&self) -> &FilesystemBackendProfile {
        &self.profile
    }

    pub fn basis(&self) -> &RootProfileQualificationBasis {
        &self.basis
    }

    pub fn qualification_report(&self) -> RootProfileQualificationReport {
        RootProfileQualificationReport::new(self.basis.binding().clone())
    }

    pub fn capabilities(&self) -> &QualifiedMediaCapabilities {
        &self.capabilities
    }

    pub(super) const fn execution_capability(&self) -> &crate::AdmittedBackendCapabilityWitness {
        &self.execution_capability
    }

    /// Produces only a capability-kind claim for scheduler planning. This does
    /// not expose the media owner, mutation lease, or execution witness.
    #[cfg(feature = "store-runtime-owner")]
    pub fn scheduler_capability_claim(
        &self,
        kind: crate::BackendCapabilityKind,
        evidence: crate::CapabilityEvidenceClass,
    ) -> Result<crate::BackendCapabilityClaimWitness, crate::BackendCapabilityAdmissionDenial> {
        self.execution_capability.require(kind, evidence)
    }

    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.store_identity
    }

    pub const fn mode(&self) -> FilesystemQualificationMode {
        self.mode
    }

    pub fn mutation_owner(&self) -> super::MutationOwnerObservation {
        self.owner.mutation_owner()
    }

    pub fn counters(&self) -> super::MediaCounterSnapshot {
        self.owner.counters()
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_next_removal_directory_sync(&self) {
        self.owner.certification_fail_next_removal_directory_sync();
    }

    pub fn counter_observer(&self) -> super::MediaCounterObserver {
        self.owner.counter_observer()
    }

    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_confinement_probe(
        &self,
        _authority: super::CertificationMediaFaultAuthority,
        component: &str,
    ) -> Result<(), super::NamespaceConfinementDenial> {
        self.owner.certification_confinement_probe(component)
    }

    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_staging_effect_probe(
        &self,
        _authority: super::CertificationMediaFaultAuthority,
        component: &str,
    ) -> super::CertificationConfinementEffect {
        self.owner.certification_staging_effect_probe(component)
    }

    pub fn close(self) -> super::OwnershipReleaseOutcome {
        let Self {
            owner,
            profile,
            basis,
            capabilities,
            execution_capability: _,
            store_identity: _,
            mode: _,
        } = self;
        drop((profile, basis, capabilities));
        owner.close()
    }

    #[cfg(test)]
    pub(super) fn into_runtime_parts(
        self,
    ) -> (
        FilesystemMediaOwner,
        FilesystemBackendProfile,
        RootProfileQualificationBasis,
        QualifiedMediaCapabilities,
        StableStoreIdentity,
    ) {
        (
            self.owner,
            self.profile,
            self.basis,
            self.capabilities,
            self.store_identity,
        )
    }
}

impl FilesystemMediaOwner {
    pub(crate) fn qualify(request: FilesystemQualificationRequest) -> AdmittedFilesystemMedia {
        let Some(access_contract) = request.access.admitted_contract() else {
            return worth_proof::TransitionOutcome::denied(
                MediaQualificationDenial::UnmanagedWriterPosture {
                    counters: Box::new(super::MediaCounterSnapshot::default()),
                },
            )
            .into();
        };
        let preflight_counters = Arc::new(super::operation_counters::MediaCounterCells::default());
        let preflight_boundary = super::fault_interposition::MediaFaultInterposer::new(
            request.fault_schedule,
            Arc::clone(&preflight_counters),
        );
        if let Some(runtime_incarnation) = request.runtime_incarnation {
            preflight_boundary.bind_runtime_incarnation(runtime_incarnation);
        }
        let preflight_profile = match super::profile_observation::observe_admission_profile(
            &request.root,
            &preflight_boundary,
        ) {
            Ok(profile) => profile,
            Err(kind) => {
                return worth_proof::TransitionOutcome::stale(
                    MediaQualificationStale::RootUnavailable {
                        kind,
                        counters: preflight_counters.snapshot(),
                    },
                )
                .into()
            }
        };
        if let Some(denial) = super::profile_observation::deny_profile(
            &preflight_profile,
            preflight_counters.snapshot(),
        ) {
            return worth_proof::TransitionOutcome::denied(denial).into();
        }
        if let Some(expected) = request.expected_basis.as_ref() {
            let binding =
                super::profile_observation::profile_binding(&preflight_profile, access_contract);
            if let Some(drift) = super::qualification_basis_drift::basis_drift(expected, &binding) {
                return super::qualification_basis_drift::pre_ownership_drift(
                    drift,
                    binding.root_identity,
                    preflight_counters.snapshot(),
                );
            }
        }
        let owner = match Self::admit_with_boundary(
            &request.root,
            preflight_boundary,
            preflight_counters,
        ) {
            Ok(owner) => owner,
            Err(failure)
                if failure.denial
                    == FilesystemMediaOwnerAdmissionDenial::Ownership(
                        super::MutationOwnershipDenial::Contended,
                    )
                    && !failure.effect_possible() =>
            {
                return worth_proof::TransitionOutcome::deferred(
                    MediaQualificationDeferred::MutationOwnerContended {
                        counters: failure.counters,
                    },
                )
                .into();
            }
            Err(failure) if !failure.effect_possible() => {
                return worth_proof::TransitionOutcome::denied(
                    MediaQualificationDenial::OwnerPreEffect {
                        denial: failure.denial,
                        release: failure.release,
                        counters: Box::new(failure.counters),
                    },
                )
                .into()
            }
            Err(failure) => {
                return worth_proof::TransitionOutcome::failed(
                    MediaQualificationFailure::OwnerAfterEffect {
                        denial: failure.denial,
                        release: failure.release,
                        counters: Box::new(failure.counters),
                    },
                )
                .into()
            }
        };
        let profile = match super::profile_observation::observe_profile(
            owner.root_directory_handle(),
            owner.boundary(),
        ) {
            Ok(profile) => profile,
            Err(kind) => {
                let cause = MediaQualificationPostOwnershipCause::ProfileObservation { kind };
                return worth_proof::TransitionOutcome::failed(close_after_ownership(owner, cause))
                    .into();
            }
        };
        if let Err(denial) = super::namespace_admission::require_opened_root_identity(
            &request.root,
            owner.root_directory_handle().directory(),
            owner.boundary(),
        ) {
            let cause = MediaQualificationPostOwnershipCause::RootIdentityChanged(denial);
            return worth_proof::TransitionOutcome::failed(close_after_ownership(owner, cause))
                .into();
        }
        if let Some(denial) = super::profile_observation::deny_profile(&profile, owner.counters()) {
            return fail_after_ownership_denial(owner, denial);
        }
        let binding = super::profile_observation::profile_binding(&profile, access_contract);
        if let Some(expected) = request.expected_basis.as_ref() {
            if let Some(drift) = super::qualification_basis_drift::basis_drift(expected, &binding) {
                let cause = MediaQualificationPostOwnershipCause::ProfileChanged { drift };
                return worth_proof::TransitionOutcome::failed(close_after_ownership(owner, cause))
                    .into();
            }
        }
        let admitted_identity =
            match super::namespace_identity_admission::admit_store_identity(&owner) {
                Ok(identity) => identity,
                Err(cause) => {
                    return worth_proof::TransitionOutcome::failed(close_after_ownership(
                        owner, cause,
                    ))
                    .into()
                }
            };
        owner
            .boundary()
            .bind_store(admitted_identity.stable_identity());
        #[cfg(any(test, feature = "certification-test-authority"))]
        if request.mode == FilesystemQualificationMode::Certification
            && super::qualification_transaction::run_bounded_qualification(&owner, &request.root)
                .is_err()
        {
            let cause = MediaQualificationPostOwnershipCause::QualificationTransaction;
            return worth_proof::TransitionOutcome::failed(close_after_ownership(owner, cause))
                .into();
        }
        #[cfg(any(test, feature = "certification-test-authority"))]
        if request.mode == FilesystemQualificationMode::Certification {
            owner.boundary().counters().qualification_transaction();
        }
        let Some(qualification) = MediaQualificationIdentity::generate() else {
            let cause = MediaQualificationPostOwnershipCause::QualificationIdentityExhausted;
            return worth_proof::TransitionOutcome::failed(close_after_ownership(owner, cause))
                .into();
        };
        let (execution_capability, buffered, file_sync, directory_sync, durable_rename) =
            match super::capability_qualification::qualify_backend_claims(
                &owner,
                &admitted_identity,
                &profile,
            ) {
                Ok(claims) => claims,
                Err(denial) => {
                    let denial = MediaQualificationDenial::Capability {
                        denial,
                        counters: Box::new(owner.counters()),
                    };
                    return fail_after_ownership_denial(owner, denial);
                }
            };
        let capabilities = QualifiedMediaCapabilities::for_observed_profile(
            qualification,
            &profile,
            buffered,
            file_sync,
            directory_sync,
            durable_rename,
        );
        let store_identity = admitted_identity.stable_identity();
        worth_proof::TransitionOutcome::success(QualifiedFilesystemMedia {
            owner,
            profile,
            basis: RootProfileQualificationBasis::new(binding),
            capabilities,
            execution_capability,
            store_identity,
            mode: request.mode,
        })
        .into()
    }
}

fn close_after_ownership(
    owner: FilesystemMediaOwner,
    cause: MediaQualificationPostOwnershipCause,
) -> MediaQualificationFailure {
    let counters = owner.counter_observer();
    let release = owner.close();
    MediaQualificationFailure::PostOwnership {
        cause: Box::new(cause),
        release,
        counters: Box::new(counters.snapshot()),
    }
}

fn fail_after_ownership_denial(
    owner: FilesystemMediaOwner,
    denial: MediaQualificationDenial,
) -> AdmittedFilesystemMedia {
    let counters = owner.counter_observer();
    let release = owner.close();
    let counters = counters.snapshot();
    worth_proof::TransitionOutcome::failed(MediaQualificationFailure::PostOwnership {
        cause: Box::new(MediaQualificationPostOwnershipCause::Denied(Box::new(
            denial.with_terminal_counters(counters),
        ))),
        release,
        counters: Box::new(counters),
    })
    .into()
}
