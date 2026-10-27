use worth_query_declaration::facade::application_resource::{
    ApplicationResourceAction as Action, ApplicationResourceDisclosureFacet as Facet,
};

/// A closed application surface, selecting an action and exactly its outputs.
/// Choosing a purpose supplies no action or disclosure permission.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryResourceAccessPurpose {
    Start,
    Advance,
    Close,
    Availability,
    State,
    Result,
    DomainFailure,
    CleanupDetail,
    Reclaim,
    CloseDetail,
}

impl WorthQueryResourceAccessPurpose {
    pub const ALL: [Self; 10] = [
        Self::Start,
        Self::Advance,
        Self::Close,
        Self::Availability,
        Self::State,
        Self::Result,
        Self::DomainFailure,
        Self::CleanupDetail,
        Self::Reclaim,
        Self::CloseDetail,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Advance => "advance",
            Self::Close => "close",
            Self::Availability => "availability",
            Self::State => "state",
            Self::Result => "result",
            Self::DomainFailure => "domain-failure",
            Self::CleanupDetail => "cleanup-detail",
            Self::Reclaim => "reclaim",
            Self::CloseDetail => "close-detail",
        }
    }

    pub const fn action(self) -> Action {
        match self {
            Self::Start => Action::Start,
            Self::Advance => Action::Advance,
            Self::Close | Self::Reclaim | Self::CloseDetail => Action::Close,
            Self::Availability
            | Self::State
            | Self::Result
            | Self::DomainFailure
            | Self::CleanupDetail => Action::Inspect,
        }
    }

    pub const fn facets(self) -> &'static [Facet] {
        match self {
            Self::Start | Self::Advance | Self::Close => &[],
            Self::Availability | Self::Reclaim => &[Facet::Availability],
            Self::State => &[Facet::State],
            Self::Result => &[Facet::Availability, Facet::Result],
            Self::DomainFailure => &[Facet::Availability, Facet::DomainFailure],
            Self::CleanupDetail | Self::CloseDetail => &[Facet::CleanupDetail],
        }
    }
}
