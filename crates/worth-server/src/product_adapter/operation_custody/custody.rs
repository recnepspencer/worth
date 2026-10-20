//! Server retains actual native owners and returns every refused owner to custody.
use super::contract::{
    WorthServerCompletedSource as Completed, WorthServerQuerySourceAdapter as SourceCommand,
    WorthServerSourceOperation as Operation, WorthServerTerminalSource as Terminal,
};
use super::view::View;
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    application_entry::{
        WorthQueryApplicationRequest, WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial, WorthQueryOperationCloseDenial,
        WorthQuerySourceOnlyAdvance, WorthQuerySourceOnlyDenial,
    },
    declaration::authentication::WorthQueryExternalPrincipalIdentity,
};
pub(super) enum Custody<C: SourceCommand> {
    Preparing(C::Preparation),
    Active(Operation<C>),
    Completed(Completed<C>),
    Terminal(Terminal<C>),
}
pub(super) struct Record<C: SourceCommand> {
    pub(super) owner: WorthQueryExternalPrincipalIdentity,
    pub(super) logical_key: String,
    pub(super) sequence: u64,
    custody: Custody<C>,
    refusal: Option<WorthQuerySourceOnlyDenial>,
    close_refusal: Option<WorthQueryOperationCloseDenial>,
}
pub(super) enum Closure {
    NativeClosed,
    PreparationDiscarded,
}
impl<C: SourceCommand> Record<C> {
    pub(super) fn new(
        owner: WorthQueryExternalPrincipalIdentity,
        logical_key: String,
        intent: C::Input,
        basis: Option<&str>,
    ) -> Box<Self> {
        Box::new(Self {
            owner,
            logical_key,
            sequence: 0,
            custody: Custody::Preparing(C::begin(intent, basis)),
            refusal: None,
            close_refusal: None,
        })
    }
    pub(super) fn advance(
        mut self: Box<Self>,
        app: &C,
        principal: &WorthQueryAuthenticatedExternalPrincipal<C::Schema>,
        scope: &WorthQueryRequestScope,
    ) -> Box<Self> {
        self.close_refusal = None;
        self.refusal = None;
        self.custody = match self.custody {
            Custody::Preparing(preparation) => {
                match app.prepare(preparation, principal, scope, &self.logical_key) {
                    Ok(owner) => Custody::Active(owner),
                    Err(preparation) => Custody::Preparing(*preparation),
                }
            }
            unchanged => unchanged,
        };
        let request = app.program().runtime().request(principal, scope);
        self.custody = match self.custody {
            Custody::Active(operation) => match operation.advance(&request) {
                WorthQuerySourceOnlyAdvance::Progress(operation) => Custody::Active(operation),
                WorthQuerySourceOnlyAdvance::Refused { operation, denial } => {
                    self.refusal = Some(denial);
                    Custody::Active(operation)
                }
                WorthQuerySourceOnlyAdvance::Completed(done) => Custody::Completed(done),
                WorthQuerySourceOnlyAdvance::Terminal(done) => Custody::Terminal(done),
            },
            unchanged => unchanged,
        };
        self
    }
    pub(super) fn close(
        mut self: Box<Self>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, C::Schema>,
    ) -> Result<Closure, Box<Self>> {
        let refusal = match self.custody {
            Custody::Preparing(_) => return Ok(Closure::PreparationDiscarded),
            Custody::Active(operation) => operation
                .close(request)
                .map(|_| ())
                .map_err(|(owner, cause)| (Custody::Active(owner), cause)),
            Custody::Completed(operation) => operation
                .close(request)
                .map(|_| ())
                .map_err(|(owner, cause)| (Custody::Completed(owner), cause)),
            Custody::Terminal(operation) => operation
                .close(request)
                .map(|_| ())
                .map_err(|(owner, cause)| (Custody::Terminal(owner), cause)),
        };
        match refusal {
            Ok(()) => Ok(Closure::NativeClosed),
            Err((custody, cause)) => {
                self.custody = custody;
                self.close_refusal = Some(cause);
                Err(self)
            }
        }
    }
    pub(super) fn assess(
        &self,
        adapter: &C,
        request: &WorthQueryApplicationRequest<'_, '_, '_, C::Schema>,
    ) -> Result<(), Option<WorthQueryApplicationRequestMutationDenial>> {
        match &self.custody {
            Custody::Preparing(preparation) => adapter
                .assess_preparation(preparation, request)
                .ok_or(None)?
                .map(|_| ())
                .map_err(Some),
            Custody::Active(owner) => owner
                .assess_current_authorization(request)
                .map(|_| ())
                .map_err(Some),
            Custody::Completed(owner) => owner
                .assess_current_authorization(request)
                .map(|_| ())
                .map_err(Some),
            Custody::Terminal(owner) => owner
                .assess_current_authorization(request)
                .map(|_| ())
                .map_err(Some),
        }
    }
    pub(super) fn view(&self) -> View {
        super::view::describe::<C>(
            &self.custody,
            self.refusal.as_ref(),
            self.close_refusal.as_ref(),
        )
    }
}
