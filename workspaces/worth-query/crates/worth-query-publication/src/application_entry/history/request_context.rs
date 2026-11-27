//! One owner for a history document's request authority. A request naming
//! another runtime is refused. An admitted request reads the document's own
//! branch, and its metadata is admitted by the installed Public contract or by
//! a fresh capability.
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::ApplicationHistoryModel,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::WorthQueryInstalledDocumentHistory;

type Denial = WorthQueryApplicationHistoryDenial;

/// A request admitted to one document's runtime.
pub(super) struct DocumentRequestContext<'a, 'p, 's, S> {
    current: WorthQueryApplicationRequest<'a, 'p, 's, S>,
}

impl<'a, 'p, 's, S: ApplicationSchema> DocumentRequestContext<'a, 'p, 's, S> {
    /// Refuse a foreign runtime; read on the document's own branch.
    pub(super) fn bind<H: ApplicationHistoryBinding<S>>(
        owner: &WorthQueryDocumentHistory<'_, S, H>,
        request: &WorthQueryApplicationRequest<'a, 'p, 's, S>,
    ) -> Result<Self, Denial> {
        if !std::ptr::eq(owner.application(), request.application) {
            return Err(Denial::ForeignRuntime);
        }
        Ok(Self {
            current: WorthQueryApplicationRequest {
                application: request.application,
                principal: request.principal,
                scope: request.scope,
                branch: owner.branch(),
            },
        })
    }

    /// A writer is never retargeted: a request on another branch is refused.
    pub(super) fn bind_exact<H: ApplicationHistoryBinding<S>>(
        owner: &WorthQueryDocumentHistory<'_, S, H>,
        request: &WorthQueryApplicationRequest<'a, 'p, 's, S>,
        program: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<Self, Denial> {
        let runtime = owner.application();
        if !std::ptr::eq(runtime, request.application) || !std::ptr::eq(runtime, program) {
            return Err(Denial::ForeignRuntime);
        }
        if request.branch != owner.branch() {
            return Err(Denial::RequestTargetMismatch);
        }
        Ok(Self {
            current: WorthQueryApplicationRequest {
                application: request.application,
                principal: request.principal,
                scope: request.scope,
                branch: request.branch,
            },
        })
    }

    pub(super) fn request(&self) -> &WorthQueryApplicationRequest<'a, 'p, 's, S> {
        &self.current
    }

    pub(super) fn advance<T>(
        &self,
        body: impl FnOnce(&WorthQueryAdvancementPhase<'_>) -> Result<T, Denial>,
    ) -> Result<T, Denial> {
        self.current
            .application
            .with_application_advancement(self.current.scope, |phase| body(&phase))
            .map_err(Denial::ExecutionRequest)?
    }

    /// Admit the document READ and its metadata inside an open phase.
    pub(super) fn admit_metadata<H, T>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: H::Access,
        payload: &impl WorthQueryHistoryPayloadMode<S, H>,
        authority: impl MetadataAuthority<S, H>,
        perform: impl for<'read> FnOnce(
            &WorthQueryPrimaryGraphApplicationRuntime<S>,
            &WorthQueryInstalledDocumentHistory<S, H>,
            WorthQueryDocumentHistoryRead<'read, S, H>,
        ) -> Result<T, WorthQueryDocumentHistoryDenial>,
    ) -> Result<T, Denial>
    where
        H: ApplicationHistoryBinding<S>,
        Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    {
        super::metadata::with_metadata_read::<S, H, _>(
            phase,
            &self.current,
            access,
            payload,
            |runtime, installed, read| {
                perform(runtime, installed, authority.admit(phase, runtime, read)?)
            },
        )
    }

    /// Open a phase, then admit the document READ and its metadata.
    pub(super) fn with_metadata<H, T>(
        &self,
        access: H::Access,
        payload: &impl WorthQueryHistoryPayloadMode<S, H>,
        authority: impl MetadataAuthority<S, H>,
        perform: impl for<'read> FnOnce(
            &WorthQueryAdvancementPhase<'_>,
            &WorthQueryPrimaryGraphApplicationRuntime<S>,
            WorthQueryDocumentHistoryRead<'read, S, H>,
        ) -> Result<T, WorthQueryDocumentHistoryDenial>,
    ) -> Result<T, Denial>
    where
        H: ApplicationHistoryBinding<S>,
        Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    {
        self.advance(|phase| {
            self.admit_metadata(phase, access, payload, authority, |runtime, _, read| {
                perform(phase, runtime, read)
            })
        })
    }
}

/// Who discloses step metadata: the installed Public contract, or a fresh
/// capability.
pub(super) trait MetadataAuthority<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> {
    fn admit<'read>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        read: WorthQueryDocumentHistoryRead<'read, S, H>,
    ) -> Result<WorthQueryDocumentHistoryRead<'read, S, H>, WorthQueryDocumentHistoryDenial>;
}

/// The installed metadata contract decides; a Governed contract refuses.
pub(super) struct Ungoverned;

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> MetadataAuthority<S, H> for Ungoverned {
    fn admit<'read>(
        self,
        _: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        read: WorthQueryDocumentHistoryRead<'read, S, H>,
    ) -> Result<WorthQueryDocumentHistoryRead<'read, S, H>, WorthQueryDocumentHistoryDenial> {
        Ok(read)
    }
}

/// A fresh metadata capability, separate from payload READ and edit authority.
pub(super) struct Governed<S, C, O, I> {
    pub(super) capability: ApplicationCapabilityRef<S, C>,
    pub(super) operation: ApplicationOperationRef<S, O, I>,
    pub(super) input: I,
}

impl<S, H, C, O, I> MetadataAuthority<S, H> for Governed<S, C, O, I>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    O: ApplicationOperationMarkerIdentity<S> + 'static,
    O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
    I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
{
    fn admit<'read>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        read: WorthQueryDocumentHistoryRead<'read, S, H>,
    ) -> Result<WorthQueryDocumentHistoryRead<'read, S, H>, WorthQueryDocumentHistoryDenial> {
        read.with_metadata_capability(runtime, phase, self.capability, self.operation, self.input)
    }
}
