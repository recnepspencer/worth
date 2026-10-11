use worth_query::facade::{
    foundation::ObservationLaneWitness,
    installed::{collection, observation},
};

#[cfg(any(test, feature = "certification-construction"))]
mod certification;
mod close;
mod open;
mod refresh;

type QueryLease = observation::WorthQuerySharedLiveProjectionLease<
    crate::WorthUiDomainEntry,
    crate::installed_domain::collection_text_projection::WorthUiCollectionTextProjection,
    crate::installed_domain::collection_text_projection::WorthUiCollectionTextProjectionFamily,
    ObservationLaneWitness,
>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiCollectionProjectionOpenStopKind {
    AlreadyOpened,
    ResourceAdmission,
    ExecutionDeferred,
    ExecutionDenied,
    ExecutionStale,
    ExecutionRebindRequired,
    ExecutionFailed,
    PublicationDenied,
    PublicationStale,
    PublicationRebindRequired,
    PublicationFailed,
    ConsumptionDenied,
    ConsumptionDeferred,
    ConsumptionStale,
    ConsumptionRebindRequired,
    ConsumptionFailed,
    SettlementDenied,
    SettlementStale,
    SettlementRebindRequired,
    SettlementFailed,
    CollectionConsumer,
    PromotionDenied,
    PromotionDeferred,
    PromotionStale,
    PromotionRebindRequired,
    PromotionAuthorityRevalidationRequired,
    PromotionFailed,
    LeaseAdmission,
}

#[must_use]
pub enum UiCollectionProjectionOpenOutcome {
    Opened(UiCollectionProjectionOpenReceipt),
    Stopped(UiCollectionProjectionOpenStop),
}

pub struct UiCollectionProjectionOpenReceipt {
    live: UiLiveCollectionProjection,
    fact: crate::UiCollectionProjectionFactReceipt,
}

pub struct UiCollectionProjectionOpenStop {
    kind: UiCollectionProjectionOpenStopKind,
    attempt_identity: worth_query::facade::runtime::WorthQueryEvidenceIdentity,
}

#[must_use = "a stopped close retains the exact live collection owner for retry"]
pub enum UiLiveCollectionProjectionCloseOutcome {
    Closed(UiLiveCollectionProjectionCloseReceipt),
    Stopped(Box<UiLiveCollectionProjectionCloseStop>),
}

pub struct UiLiveCollectionProjectionCloseReceipt {
    owner_terminal: bool,
    counters: worth_query::facade::runtime::WorthQuerySharedLeaseReleaseCounters,
}

pub struct UiLiveCollectionProjectionCloseStop {
    live: UiLiveCollectionProjection,
    query_error: worth_query::facade::runtime::WorthQueryRuntimeError,
    counters: worth_query::facade::runtime::WorthQuerySharedLeaseReleaseCounters,
}

pub enum UiCollectionProjectionRefreshError {
    Drain(Box<observation::WorthQuerySharedProjectionDrainStop>),
    Delta(Box<observation::WorthQueryConsumerInvalidationDeltaStop>),
    Readmission(Box<observation::WorthQueryConsumerInvalidationAdmissionStop>),
    Delivery(Box<collection::WorthQueryCollectionDeliveryDenial>),
}

#[must_use]
#[derive(Debug)]
pub enum UiCollectionProjectionRefreshOutcome {
    NoSemanticDelivery,
    Applied(UiCollectionProjectionRefreshReceipt),
}

#[must_use]
#[derive(Debug)]
pub struct UiCollectionProjectionRefreshReceipt {
    fact: crate::UiCollectionProjectionFactReceipt,
    query_work: crate::WorthUiCollectionQueryWorkInspection,
}

#[must_use = "a live collection projection owns a Query lease until explicitly closed"]
pub struct UiLiveCollectionProjection {
    binding: crate::UiCollectionProjectionBinding,
    reference: crate::application_binding::WorthUiInstalledCollectionTextOperationReference,
    lease: QueryLease,
    consumer: collection::WorthQueryCollectionConsumerWindow,
    text_accesses: Box<[crate::application_binding::WorthUiCollectionTextNativeAccess]>,
    application_item_key_access:
        Option<crate::application_binding::WorthUiCollectionTextNativeAccess>,
    budget: crate::UiCollectionProjectionBudget,
}

impl crate::UiCollectionProjectionBinding {
    pub fn open(
        self,
        phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        budget: crate::UiCollectionProjectionBudget,
        workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    ) -> UiCollectionProjectionOpenOutcome {
        open::open_collection_projection(phase, self, budget, workspace)
    }
}

impl UiCollectionProjectionOpenReceipt {
    pub fn fact(&self) -> &crate::UiCollectionProjectionFactReceipt {
        &self.fact
    }

    pub fn live(&self) -> &UiLiveCollectionProjection {
        &self.live
    }

    pub fn into_parts(
        self,
    ) -> (
        UiLiveCollectionProjection,
        crate::UiCollectionProjectionFactReceipt,
    ) {
        (self.live, self.fact)
    }
}

impl UiCollectionProjectionOpenStop {
    pub fn kind(&self) -> UiCollectionProjectionOpenStopKind {
        self.kind
    }

    pub fn attempt_identity(&self) -> &worth_query::facade::runtime::WorthQueryEvidenceIdentity {
        &self.attempt_identity
    }
}

impl UiLiveCollectionProjection {
    pub fn binding(&self) -> &crate::UiCollectionProjectionBinding {
        &self.binding
    }

    pub fn is_current_installation(&self) -> bool {
        self.reference.installation_is_current()
    }
}

impl UiLiveCollectionProjectionCloseReceipt {
    pub fn owner_terminal(&self) -> bool {
        self.owner_terminal
    }

    pub fn counters(&self) -> worth_query::facade::runtime::WorthQuerySharedLeaseReleaseCounters {
        self.counters
    }
}

impl UiLiveCollectionProjectionCloseStop {
    pub fn query_error(&self) -> &worth_query::facade::runtime::WorthQueryRuntimeError {
        &self.query_error
    }

    pub fn counters(&self) -> worth_query::facade::runtime::WorthQuerySharedLeaseReleaseCounters {
        self.counters
    }

    pub fn into_live(self) -> UiLiveCollectionProjection {
        self.live
    }
}

impl UiCollectionProjectionRefreshReceipt {
    pub fn fact(&self) -> &crate::UiCollectionProjectionFactReceipt {
        &self.fact
    }

    pub fn query_work(&self) -> &crate::WorthUiCollectionQueryWorkInspection {
        &self.query_work
    }

    pub fn into_fact(self) -> crate::UiCollectionProjectionFactReceipt {
        self.fact
    }
}

impl std::fmt::Debug for UiCollectionProjectionOpenOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Opened(_) => formatter.write_str("Opened(exact live collection owner)"),
            Self::Stopped(stop) => formatter.debug_tuple("Stopped").field(stop).finish(),
        }
    }
}

impl std::fmt::Debug for UiCollectionProjectionOpenStop {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UiCollectionProjectionOpenStop")
            .field("kind", &self.kind)
            .field("attempt_identity_scope", &self.attempt_identity.scope())
            .field("attempt_identity_scheme", &self.attempt_identity.scheme())
            .finish()
    }
}

fn stopped(
    binding: &crate::UiCollectionProjectionBinding,
    kind: UiCollectionProjectionOpenStopKind,
) -> UiCollectionProjectionOpenOutcome {
    UiCollectionProjectionOpenOutcome::Stopped(UiCollectionProjectionOpenStop {
        kind,
        attempt_identity: binding.query_world_identity().clone(),
    })
}
