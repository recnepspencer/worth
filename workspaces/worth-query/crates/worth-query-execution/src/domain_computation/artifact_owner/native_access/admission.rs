use std::collections::BTreeSet;
use std::sync::Arc;

use worth_foundational::facade::AspectKey;
use worth_query_installation::facade::{
    WorthQueryArtifactNativeAccessContract, WorthQueryArtifactNativeLayoutReference,
    WorthQueryInstalledArtifactContractAuthority,
};

use super::super::{
    WorthQueryArtifactAuthorityMatch, WorthQueryArtifactDenialKind, WorthQueryBorrowedArtifactView,
    WorthQueryRuntimeArtifactOwner, WorthQueryTransferredArtifactHandle,
};
use super::denial::denial_detail;
use super::thread_bound::WorthQueryArtifactThreadBound;
use super::{
    WorthQueryArtifactNativeAccessBound, WorthQueryArtifactNativeAccessCounters,
    WorthQueryArtifactNativeAccessDenial, WorthQueryArtifactNativeAccessDenialKind as Kind,
    WorthQueryArtifactNativeAccessEvidence, WorthQueryArtifactProviderAccessDenial,
    WorthQueryArtifactProviderAccessSession,
};

pub struct WorthQueryArtifactAccessAuthority {
    pub(super) contract: Arc<WorthQueryInstalledArtifactContractAuthority>,
    pub(super) domain_authority:
        Arc<crate::domain_computation::vocabulary::operation_binding::installed_domain_authority::WorthQueryInstalledDomainExecutionAuthority>,
    pub(super) operation_identity: String,
    pub(super) binding_identity: String,
    pub(super) run_identity: String,
    pub(super) stage_identity: String,
    pub(super) basis_identity: String,
}

impl WorthQueryArtifactAccessAuthority {
    pub(crate) fn mint(
        contract: Arc<WorthQueryInstalledArtifactContractAuthority>,
        domain_authority: Arc<
            crate::domain_computation::vocabulary::operation_binding::installed_domain_authority::WorthQueryInstalledDomainExecutionAuthority,
        >,
        operation_identity: &str,
        binding_identity: &str,
        run_identity: &str,
        stage_identity: &str,
        basis_identity: &str,
    ) -> Arc<Self> {
        Arc::new(Self {
            contract,
            domain_authority,
            operation_identity: operation_identity.to_owned(),
            binding_identity: binding_identity.to_owned(),
            run_identity: run_identity.to_owned(),
            stage_identity: stage_identity.to_owned(),
            basis_identity: basis_identity.to_owned(),
        })
    }
}

pub(crate) struct WorthQueryArtifactReaderAuthorityAdmission<'a> {
    owner: &'a Arc<WorthQueryRuntimeArtifactOwner>,
    access_authority: &'a WorthQueryArtifactAccessAuthority,
    native_contract: &'a WorthQueryArtifactNativeAccessContract,
    counters: WorthQueryArtifactNativeAccessCounters,
}

impl<'a> WorthQueryArtifactReaderAuthorityAdmission<'a> {
    pub(crate) fn admit(
        handle: &'a WorthQueryTransferredArtifactHandle,
        authority: &'a WorthQueryArtifactAccessAuthority,
    ) -> Result<Self, WorthQueryArtifactNativeAccessDenial> {
        let owner = &handle.core.owner;
        let binding = owner.binding();
        let mut counters = WorthQueryArtifactNativeAccessCounters::default();
        let authority_match = WorthQueryArtifactAuthorityMatch {
            runtime: checked(
                &mut counters,
                binding.domain_authority.runtime_authority()
                    == authority.domain_authority.runtime_authority(),
            ),
            generation: checked(
                &mut counters,
                binding
                    .domain_authority
                    .is_current_installation_generation()
                    && authority
                        .domain_authority
                        .is_current_installation_generation()
                    && binding.domain_authority.installation_generation()
                        == authority.domain_authority.installation_generation(),
            ),
            operation: checked(
                &mut counters,
                binding.operation_identity == authority.operation_identity
                    && binding.binding_identity == authority.binding_identity,
            ),
            run: checked(
                &mut counters,
                binding.run_identity == authority.run_identity,
            ),
            stage: checked(
                &mut counters,
                handle.core.holder_stage == authority.stage_identity,
            ),
            basis: checked(
                &mut counters,
                binding.basis_identity == authority.basis_identity,
            ),
            payload_owner: checked(
                &mut counters,
                binding.contract.owner() == authority.contract.owner(),
            ),
            contract: checked(
                &mut counters,
                binding.contract.contract().identity() == authority.contract.contract().identity(),
            ),
        };
        if let Some(kind) = authority_match.denial_kind() {
            return Err(denial(owner, map_authority_denial(kind), counters));
        }
        counters.authority_checks += 1;
        if owner.created_thread() != std::thread::current().id() {
            return Err(denial(owner, Kind::ForeignThread, counters));
        }
        let Some(native_contract) = authority
            .contract
            .contract()
            .access_path()
            .native_contract()
        else {
            return Err(denial(owner, Kind::AccessPathDenied, counters));
        };
        Ok(Self {
            owner,
            access_authority: authority,
            native_contract,
            counters,
        })
    }

    pub(crate) fn native_contract(&self) -> &WorthQueryArtifactNativeAccessContract {
        self.native_contract
    }

    pub(crate) fn denial(&self, kind: Kind) -> WorthQueryArtifactNativeAccessDenial {
        denial(self.owner, kind, self.counters)
    }

    pub(crate) fn admit_access(
        self,
        handle: &'a WorthQueryTransferredArtifactHandle,
        layout: &WorthQueryArtifactNativeLayoutReference,
        fields: &[AspectKey],
        access_bound: WorthQueryArtifactNativeAccessBound,
    ) -> Result<WorthQueryArtifactNativeAccessAdmission<'a>, WorthQueryArtifactNativeAccessDenial>
    {
        let mut counters = self.counters;
        counters.layout_checks += 1;
        if self.native_contract.layout().reference() != *layout {
            return Err(denial(self.owner, Kind::LayoutMismatch, counters));
        }
        if fields.is_empty() {
            return Err(denial(self.owner, Kind::FieldNotDeclared, counters));
        }
        let declared = self
            .native_contract
            .layout()
            .fields()
            .iter()
            .map(|field| field.aspect().key())
            .collect::<BTreeSet<_>>();
        let mut requested = BTreeSet::new();
        for field in fields {
            counters.requested_field_checks += 1;
            if !declared.contains(field) || !requested.insert(field) {
                return Err(denial(self.owner, Kind::FieldNotDeclared, counters));
            }
        }
        let borrowed = WorthQueryBorrowedArtifactView::admit(
            self.owner,
            handle.core.guard,
            "native-artifact-access",
        )
        .map_err(|artifact| {
            counters.lifecycle_checks += 1;
            denial(self.owner, map_lifecycle_denial(artifact.kind()), counters)
        })?;
        counters.lifecycle_checks += 1;
        let session = WorthQueryArtifactProviderAccessSession::mint(
            self.owner.provider_access_session_identity().to_owned(),
            1,
            borrowed.borrow_generation(),
            layout.clone(),
        );
        Ok(WorthQueryArtifactNativeAccessAdmission {
            owner: self.owner,
            borrowed,
            native_contract: self.native_contract,
            basis_identity: self.access_authority.basis_identity.clone(),
            requested_fields: fields.to_vec(),
            access_bound,
            session,
            counters,
            _thread_bound: WorthQueryArtifactThreadBound::new(),
        })
    }
}

pub(crate) struct WorthQueryArtifactNativeAccessAdmission<'a> {
    owner: &'a Arc<WorthQueryRuntimeArtifactOwner>,
    borrowed: WorthQueryBorrowedArtifactView<'a>,
    native_contract: &'a WorthQueryArtifactNativeAccessContract,
    basis_identity: String,
    requested_fields: Vec<AspectKey>,
    access_bound: WorthQueryArtifactNativeAccessBound,
    session: WorthQueryArtifactProviderAccessSession,
    counters: WorthQueryArtifactNativeAccessCounters,
    _thread_bound: WorthQueryArtifactThreadBound,
}

impl WorthQueryArtifactNativeAccessAdmission<'_> {
    pub(crate) fn native_contract(&self) -> &WorthQueryArtifactNativeAccessContract {
        self.native_contract
    }

    pub(crate) fn requested_fields(&self) -> &[AspectKey] {
        &self.requested_fields
    }

    pub(crate) fn counters_mut(&mut self) -> &mut WorthQueryArtifactNativeAccessCounters {
        &mut self.counters
    }

    pub(crate) fn denial(
        &self,
        kind: Kind,
        detail: &'static str,
    ) -> WorthQueryArtifactNativeAccessDenial {
        WorthQueryArtifactNativeAccessDenial::new(
            kind,
            Some(self.owner.binding().contract.contract().family().as_str()),
            detail,
            self.counters,
        )
    }

    pub(crate) fn with_provider<T>(
        &mut self,
        access: impl FnOnce(
            &dyn super::WorthQueryArtifactNativeAccessProvider,
            &WorthQueryArtifactProviderAccessSession,
        ) -> Result<T, WorthQueryArtifactProviderAccessDenial>,
    ) -> Result<T, WorthQueryArtifactNativeAccessDenial> {
        if self.owner.created_thread() != std::thread::current().id() {
            return Err(denial(self.owner, Kind::ForeignThread, self.counters));
        }
        self.owner
            .validate_borrow_generation(self.borrowed.borrow_generation())
            .map_err(|artifact| {
                self.counters.lifecycle_checks += 1;
                denial(
                    self.owner,
                    map_lifecycle_denial(artifact.kind()),
                    self.counters,
                )
            })?;
        self.counters.lifecycle_checks += 1;
        self.counters.provider_session_checks += 1;
        if self.session.identity() != self.owner.provider_access_session_identity()
            || self.session.generation() != 1
            || self.session.borrow_generation() != self.borrowed.borrow_generation()
            || self.session.layout() != &self.native_contract.layout().reference()
        {
            return Err(denial(
                self.owner,
                Kind::ProviderSessionMismatch,
                self.counters,
            ));
        }
        self.counters.provider_contacts += 1;
        let session = self.session.clone();
        let expected_layout = session.layout().clone();
        let outcome = self
            .owner
            .with_native_access_provider(|provider| {
                self.counters.provider_session_checks += 1;
                let provider_layout = provider.layout();
                if provider_layout.identity() != expected_layout.identity()
                    || provider_layout.version() != expected_layout.version()
                {
                    return Err(WorthQueryArtifactProviderAccessDenial::LayoutMismatch);
                }
                if provider_layout.alignment() != expected_layout.alignment() {
                    return Err(WorthQueryArtifactProviderAccessDenial::AlignmentMismatch);
                }
                access(provider, &session)
            })
            .ok_or_else(|| denial(self.owner, Kind::NativeProviderUnavailable, self.counters))?;
        outcome.map_err(|provider| {
            let kind = match provider {
                WorthQueryArtifactProviderAccessDenial::SessionMismatch => {
                    Kind::ProviderSessionMismatch
                }
                WorthQueryArtifactProviderAccessDenial::LayoutMismatch => Kind::LayoutMismatch,
                WorthQueryArtifactProviderAccessDenial::AlignmentMismatch => {
                    Kind::AlignmentMismatch
                }
                WorthQueryArtifactProviderAccessDenial::BoundsExceeded => Kind::BoundsExceeded,
                WorthQueryArtifactProviderAccessDenial::ShapeMismatch => {
                    Kind::ProviderShapeMismatch
                }
                WorthQueryArtifactProviderAccessDenial::NoProgress => Kind::ProviderDenied,
                WorthQueryArtifactProviderAccessDenial::Unsupported
                | WorthQueryArtifactProviderAccessDenial::Failed => Kind::ProviderDenied,
            };
            denial(self.owner, kind, self.counters)
        })
    }

    pub(crate) fn evidence(&self) -> WorthQueryArtifactNativeAccessEvidence {
        WorthQueryArtifactNativeAccessEvidence::new(
            self.owner.binding().occurrence_identity.clone(),
            self.basis_identity.clone(),
            self.session.identity().to_owned(),
            self.session.layout().clone(),
            self.requested_fields.clone(),
            self.access_bound.clone(),
            self.borrowed.borrow_generation(),
            self.counters,
        )
    }
}

fn checked(counters: &mut WorthQueryArtifactNativeAccessCounters, matches: bool) -> bool {
    counters.authority_checks += 1;
    matches
}

fn denial(
    owner: &WorthQueryRuntimeArtifactOwner,
    kind: Kind,
    counters: WorthQueryArtifactNativeAccessCounters,
) -> WorthQueryArtifactNativeAccessDenial {
    WorthQueryArtifactNativeAccessDenial::new(
        kind,
        Some(owner.binding().contract.contract().family().as_str()),
        denial_detail(kind),
        counters,
    )
}

fn map_authority_denial(kind: WorthQueryArtifactDenialKind) -> Kind {
    match kind {
        WorthQueryArtifactDenialKind::ForeignRuntime => Kind::ForeignRuntime,
        WorthQueryArtifactDenialKind::StaleInstallationGeneration => {
            Kind::StaleInstallationGeneration
        }
        WorthQueryArtifactDenialKind::OperationMismatch => Kind::OperationMismatch,
        WorthQueryArtifactDenialKind::RunMismatch => Kind::RunMismatch,
        WorthQueryArtifactDenialKind::StageMismatch => Kind::StageMismatch,
        WorthQueryArtifactDenialKind::BasisMismatch => Kind::BasisMismatch,
        WorthQueryArtifactDenialKind::PayloadOwnerMismatch => Kind::PayloadOwnerMismatch,
        _ => Kind::ArtifactContractMismatch,
    }
}

fn map_lifecycle_denial(kind: WorthQueryArtifactDenialKind) -> Kind {
    match kind {
        WorthQueryArtifactDenialKind::AlreadyDisposed => Kind::AlreadyDisposed,
        WorthQueryArtifactDenialKind::StaleLifecycleGeneration => Kind::StaleBorrowGeneration,
        _ => Kind::ProviderDenied,
    }
}
