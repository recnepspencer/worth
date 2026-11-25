//! Owned `'static` document history attachment with full head custody preservation.
use std::borrow::Borrow;
use std::fmt;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding,
    application_program::ApplicationProgramDefinition,
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementDenial,
    primary_graph::WorthQueryApplicationProjection, product::WorthQueryOwnedProductHead,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationHistoryDenial, WorthQueryApplicationRequestExt,
    WorthQueryOwnedApplicationHistoryDocument,
};

use super::super::preview::{fork_expected_from_active_with_scope, WorthQueryPreviewForkDenial};
use super::WorthQueryApplicationHome;

type HistoryAccessBinding<S, H> =
    <<H as ApplicationHistoryBinding<S>>::Access as ApplicationQueryIntent<S>>::Binding;
type HistoryQuery<S, H> = <HistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query;
type HistoryScope<S, H> = <<HistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;
type HistoryPrincipal<S, H> =
    <HistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::PrincipalIdentity;
type HistoryRows<S, H> = <<HistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// Why attaching an owned `'static` history document was refused, preserving
/// exact owned-head custody when attachment or advancement fails after forking.
#[must_use = "retain or close any owned head returned on history attachment failure"]
pub enum WorthQueryHomeOwnedHistoryDenial<'home, Schema> {
    Fork(WorthQueryPreviewForkDenial),
    Advancement {
        denial: WorthQueryAdvancementDenial,
        head: Box<WorthQueryOwnedProductHead<'home, Schema>>,
    },
    AdvancementAfterConsume(WorthQueryAdvancementDenial),
    Attachment {
        denial: WorthQueryApplicationHistoryDenial,
        head: Box<WorthQueryOwnedProductHead<'home, Schema>>,
    },
}

impl<Schema> fmt::Debug for WorthQueryHomeOwnedHistoryDenial<'_, Schema> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fork(denial) => f.debug_tuple("Fork").field(denial).finish(),
            Self::Advancement { denial, head } => f
                .debug_struct("Advancement")
                .field("denial", denial)
                .field("branch", &head.branch())
                .finish(),
            Self::AdvancementAfterConsume(denial) => f
                .debug_tuple("AdvancementAfterConsume")
                .field(denial)
                .finish(),
            Self::Attachment { denial, head } => f
                .debug_struct("Attachment")
                .field("denial", denial)
                .field("branch", &head.branch())
                .finish(),
        }
    }
}

impl<'home, Schema> WorthQueryHomeOwnedHistoryDenial<'home, Schema> {
    /// Extracts the retained owned head when failure occurred after forking.
    pub fn into_owned_head(self) -> Option<WorthQueryOwnedProductHead<'home, Schema>> {
        match self {
            Self::Advancement { head, .. } | Self::Attachment { head, .. } => Some(*head),
            Self::Fork(_) | Self::AdvancementAfterConsume(_) => None,
        }
    }
}

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    /// Forks a dedicated head from the Home's active branch and attaches an
    /// owned `'static` [`WorthQueryOwnedApplicationHistoryDocument`].
    ///
    /// Because the returned document is `'static`, applications can store it
    /// directly in desktop UI state alongside the Home without lifetime borrows.
    /// Release or close the document before closing the Home.
    pub fn owned_history<'home, H>(
        &'home self,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        access: H::Access,
    ) -> Result<
        WorthQueryOwnedApplicationHistoryDocument<Schema, H>,
        WorthQueryHomeOwnedHistoryDenial<'home, Schema>,
    >
    where
        H: ApplicationHistoryBinding<Schema>,
        <HistoryAccessBinding<Schema, H> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, HistoryPrincipal<Schema, H>>,
        HistoryRows<Schema, H>: WorthQueryApplicationProjection<Schema, HistoryQuery<Schema, H>>,
        HistoryScope<Schema, H>: 'static,
    {
        let principal = principal.borrow();
        let scope = WorthQueryRequestScope::new(
            None,
            WorthQueryCancellationSource::new().token(),
            principal.clock().clone(),
        );
        self.owned_history_with_scope::<H>(principal, access, &scope)
    }

    /// Forks a dedicated head and attaches an owned `'static` history document
    /// under an explicit `scope`, retaining the owned head intact if attachment
    /// or advancement refuses after forking.
    pub fn owned_history_with_scope<'home, H>(
        &'home self,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        access: H::Access,
        scope: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryOwnedApplicationHistoryDocument<Schema, H>,
        WorthQueryHomeOwnedHistoryDenial<'home, Schema>,
    >
    where
        H: ApplicationHistoryBinding<Schema>,
        <HistoryAccessBinding<Schema, H> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, HistoryPrincipal<Schema, H>>,
        HistoryRows<Schema, H>: WorthQueryApplicationProjection<Schema, HistoryQuery<Schema, H>>,
        HistoryScope<Schema, H>: 'static,
    {
        let expected = fork_expected_from_active_with_scope(self, scope)
            .map_err(WorthQueryHomeOwnedHistoryDenial::Fork)?;
        self.attach_owned_history_head::<H>(expected.into_owned_head(), principal, access, scope)
    }

    /// Attaches an already-forked [`WorthQueryOwnedProductHead`] as an owned
    /// `'static` history document under `scope`, preserving the head intact on
    /// pre-entry advancement or attachment refusal so callers can retry or close it.
    pub fn attach_owned_history_head<'home, H>(
        &'home self,
        head: WorthQueryOwnedProductHead<'home, Schema>,
        principal: impl Borrow<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
        access: H::Access,
        scope: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryOwnedApplicationHistoryDocument<Schema, H>,
        WorthQueryHomeOwnedHistoryDenial<'home, Schema>,
    >
    where
        H: ApplicationHistoryBinding<Schema>,
        <HistoryAccessBinding<Schema, H> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, HistoryPrincipal<Schema, H>>,
        HistoryRows<Schema, H>: WorthQueryApplicationProjection<Schema, HistoryQuery<Schema, H>>,
        HistoryScope<Schema, H>: 'static,
    {
        let principal = principal.borrow();
        let graph = self.primary_graph();
        let mut head_slot = Some(head);
        let attached = graph.with_funded_application_completion(
            scope,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            |phase| {
                let head = head_slot.take().expect("advancement runs once");
                let request = graph.request(principal, scope).on_branch(head.branch());
                match request.attach_owned_history::<H, Program>(
                    self.runtime(),
                    &phase,
                    head,
                    access,
                ) {
                    Ok(document) => Ok(document),
                    Err(failure) => {
                        let (head, denial) = failure.into_parts();
                        Err(WorthQueryHomeOwnedHistoryDenial::Attachment {
                            denial,
                            head: Box::new(head),
                        })
                    }
                }
            },
        );
        match attached {
            Ok(result) => result,
            Err(denial) => match head_slot.take() {
                Some(head) => Err(WorthQueryHomeOwnedHistoryDenial::Advancement {
                    denial,
                    head: Box::new(head),
                }),
                None => Err(WorthQueryHomeOwnedHistoryDenial::AdvancementAfterConsume(
                    denial,
                )),
            },
        }
    }
}
