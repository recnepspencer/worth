use std::marker::PhantomData;

use crate::portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity};

use super::super::{ApplicationQueryObservableInfluence, ApplicationQueryResultSlotKey};

/// These surfaces are admitted independently of any payload-field visibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationHistoryMetadataFacet {
    Occurrence,
    Ancestry,
    Continuation,
}

impl ApplicationHistoryMetadataFacet {
    pub const ALL: [Self; 3] = [Self::Occurrence, Self::Ancestry, Self::Continuation];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Occurrence => "history-occurrence",
            Self::Ancestry => "history-ancestry",
            Self::Continuation => "history-continuation",
        }
    }

    pub const fn influence(self) -> ApplicationQueryObservableInfluence {
        match self {
            Self::Occurrence | Self::Ancestry => {
                ApplicationQueryObservableInfluence::HistoricalMembership
            }
            Self::Continuation => ApplicationQueryObservableInfluence::Pagination,
        }
    }
}

/// Portable, authority-free identity of one history metadata slot.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationHistoryMetadataMeaning {
    history: WorthQueryPortableTypeIdentity,
    facet: ApplicationHistoryMetadataFacet,
}

impl ApplicationHistoryMetadataMeaning {
    pub fn from_untrusted_fields(
        history: WorthQueryPortableTypeIdentity,
        facet: ApplicationHistoryMetadataFacet,
    ) -> Self {
        Self { history, facet }
    }

    pub fn history_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.history.clone()
    }

    pub const fn history_type(&self) -> &str {
        self.history.as_str()
    }

    pub const fn facet(&self) -> ApplicationHistoryMetadataFacet {
        self.facet
    }

    pub const fn slot_type(&self) -> &'static str {
        self.facet.name()
    }

    pub fn slot_key(&self) -> ApplicationQueryResultSlotKey {
        ApplicationQueryResultSlotKey::history_metadata(self.history.clone(), self.facet)
    }
}

/// A slot belongs to the exact declared History marker, never its SubjectRead.
pub struct ApplicationHistoryMetadataRef<History> {
    facet: ApplicationHistoryMetadataFacet,
    _history: PhantomData<fn() -> History>,
}

impl<History: WorthQueryPortableType> ApplicationHistoryMetadataRef<History> {
    pub const fn new(facet: ApplicationHistoryMetadataFacet) -> Self {
        Self {
            facet,
            _history: PhantomData,
        }
    }

    pub fn meaning(&self) -> ApplicationHistoryMetadataMeaning {
        ApplicationHistoryMetadataMeaning::from_untrusted_fields(
            History::PORTABLE_TYPE_IDENTITY,
            self.facet,
        )
    }
}
