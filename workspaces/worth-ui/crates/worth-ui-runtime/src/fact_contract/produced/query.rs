#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiQueryIncrementalChangedFact {
    graph_effects: usize,
    measurement_effects: usize,
    allocation_effects: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiQueryResetChangedFact {
    reason: worth_ui_query_binding::WorthUiCollectionResetReason,
    fresh_execution_required: bool,
    maximum_replacement_rows: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiQueryChangedFactKind {
    Incremental(UiQueryIncrementalChangedFact),
    Reset(UiQueryResetChangedFact),
    ScalarProjection,
    CollectionProjection { changed_rows: usize },
}

pub struct UiQueryChangedFact {
    kind: UiQueryChangedFactKind,
    payload: UiQueryChangedFactPayload,
}

enum UiQueryChangedFactPayload {
    OperationLive(worth_ui_query_binding::WorthUiCollectionChangeConsequence),
    Projection(worth_ui_query_binding::UiProjectionObservation),
}

impl UiQueryChangedFact {
    pub(crate) fn from_owner_consequence(
        consequence: worth_ui_query_binding::WorthUiCollectionChangeConsequence,
    ) -> Self {
        let kind = match consequence.kind() {
            worth_ui_query_binding::WorthUiCollectionChangeKind::Incremental(incremental) => {
                UiQueryChangedFactKind::Incremental(UiQueryIncrementalChangedFact {
                    graph_effects: incremental.graph().len(),
                    measurement_effects: incremental.measurement().len(),
                    allocation_effects: incremental.allocation().len(),
                })
            }
            worth_ui_query_binding::WorthUiCollectionChangeKind::Reset(reset) => {
                UiQueryChangedFactKind::Reset(UiQueryResetChangedFact {
                    reason: reset.reason(),
                    fresh_execution_required: reset.fresh_execution_required(),
                    maximum_replacement_rows: reset.maximum_replacement_rows(),
                })
            }
        };
        Self {
            kind,
            payload: UiQueryChangedFactPayload::OperationLive(consequence),
        }
    }

    pub(crate) fn from_projection_observation(
        observation: worth_ui_query_binding::UiProjectionObservation,
    ) -> Self {
        let kind = match &observation {
            worth_ui_query_binding::UiProjectionObservation::Scalar(_) => {
                UiQueryChangedFactKind::ScalarProjection
            }
            worth_ui_query_binding::UiProjectionObservation::ApplicationScalar(_) => {
                UiQueryChangedFactKind::ScalarProjection
            }
            worth_ui_query_binding::UiProjectionObservation::Collection(observation) => {
                UiQueryChangedFactKind::CollectionProjection {
                    changed_rows: observation.fact().changes().len(),
                }
            }
        };
        Self {
            kind,
            payload: UiQueryChangedFactPayload::Projection(observation),
        }
    }

    pub const fn kind(&self) -> UiQueryChangedFactKind {
        self.kind
    }

    pub fn change_order(&self) -> u64 {
        match &self.payload {
            UiQueryChangedFactPayload::OperationLive(consequence) => consequence.change_order(),
            UiQueryChangedFactPayload::Projection(observation) => observation.owner_order(),
        }
    }

    pub fn projection_identity(&self) -> Option<&worth_ui_query_binding::WorthUiQueryViewIdentity> {
        match &self.payload {
            UiQueryChangedFactPayload::OperationLive(_) => None,
            UiQueryChangedFactPayload::Projection(observation) => {
                Some(observation.projection_identity())
            }
        }
    }

    pub fn scalar_projection(
        &self,
    ) -> Option<&worth_ui_query_binding::UiScalarProjectionFactReceipt> {
        match &self.payload {
            UiQueryChangedFactPayload::Projection(
                worth_ui_query_binding::UiProjectionObservation::Scalar(observation),
            ) => Some(observation.fact()),
            UiQueryChangedFactPayload::OperationLive(_)
            | UiQueryChangedFactPayload::Projection(
                worth_ui_query_binding::UiProjectionObservation::ApplicationScalar(_),
            )
            | UiQueryChangedFactPayload::Projection(
                worth_ui_query_binding::UiProjectionObservation::Collection(_),
            ) => None,
        }
    }

    pub fn application_scalar_projection(
        &self,
    ) -> Option<&worth_ui_query_binding::UiApplicationScalarProjectionFactReceipt> {
        match &self.payload {
            UiQueryChangedFactPayload::Projection(
                worth_ui_query_binding::UiProjectionObservation::ApplicationScalar(observation),
            ) => Some(observation.fact()),
            _ => None,
        }
    }

    pub fn collection_projection(
        &self,
    ) -> Option<&worth_ui_query_binding::UiCollectionProjectionFactReceipt> {
        match &self.payload {
            UiQueryChangedFactPayload::Projection(
                worth_ui_query_binding::UiProjectionObservation::Collection(observation),
            ) => Some(observation.fact()),
            UiQueryChangedFactPayload::OperationLive(_)
            | UiQueryChangedFactPayload::Projection(
                worth_ui_query_binding::UiProjectionObservation::ApplicationScalar(_),
            )
            | UiQueryChangedFactPayload::Projection(
                worth_ui_query_binding::UiProjectionObservation::Scalar(_),
            ) => None,
        }
    }

    pub(crate) fn into_application_scalar_projection(
        self,
    ) -> Result<worth_ui_query_binding::UiApplicationScalarProjectionFactReceipt, Box<Self>> {
        match self.payload {
            UiQueryChangedFactPayload::Projection(
                worth_ui_query_binding::UiProjectionObservation::ApplicationScalar(observation),
            ) => Ok(observation.into_fact()),
            payload => Err(Box::new(Self {
                kind: self.kind,
                payload,
            })),
        }
    }

    pub(crate) fn into_scalar_projection(
        self,
    ) -> Result<worth_ui_query_binding::UiScalarProjectionFactReceipt, Box<Self>> {
        match self.payload {
            UiQueryChangedFactPayload::Projection(
                worth_ui_query_binding::UiProjectionObservation::Scalar(observation),
            ) => Ok(observation.into_fact()),
            payload => Err(Box::new(Self {
                kind: self.kind,
                payload,
            })),
        }
    }

    pub(crate) fn into_owner_consequence(
        self,
    ) -> Result<worth_ui_query_binding::WorthUiCollectionChangeConsequence, Box<Self>> {
        match self.payload {
            UiQueryChangedFactPayload::OperationLive(consequence) => Ok(consequence),
            payload => Err(Box::new(Self {
                kind: self.kind,
                payload,
            })),
        }
    }

    pub(crate) fn into_projection_observation(
        self,
    ) -> Result<worth_ui_query_binding::UiProjectionObservation, Box<Self>> {
        match self.payload {
            UiQueryChangedFactPayload::Projection(observation) => Ok(observation),
            payload => Err(Box::new(Self {
                kind: self.kind,
                payload,
            })),
        }
    }
}

impl UiQueryIncrementalChangedFact {
    pub const fn graph_effects(self) -> usize {
        self.graph_effects
    }

    pub const fn measurement_effects(self) -> usize {
        self.measurement_effects
    }

    pub const fn allocation_effects(self) -> usize {
        self.allocation_effects
    }
}

impl UiQueryResetChangedFact {
    pub const fn reason(self) -> worth_ui_query_binding::WorthUiCollectionResetReason {
        self.reason
    }

    pub const fn fresh_execution_required(self) -> bool {
        self.fresh_execution_required
    }

    pub const fn maximum_replacement_rows(self) -> usize {
        self.maximum_replacement_rows
    }
}
