use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryObservableInfluence {
    RowPresence,
    Ordering,
    Pagination,
    Count,
    Aggregate,
    Explanation,
    HistoricalMembership,
    Preview,
    LiveMembership,
}

impl ApplicationQueryObservableInfluence {
    /// The complete closed vocabulary used by disclosure and its work bounds.
    pub const ALL: [Self; 9] = [
        Self::RowPresence,
        Self::Ordering,
        Self::Pagination,
        Self::Count,
        Self::Aggregate,
        Self::Explanation,
        Self::HistoricalMembership,
        Self::Preview,
        Self::LiveMembership,
    ];
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryInfluenceContract {
    permitted: BTreeSet<ApplicationQueryObservableInfluence>,
}

impl ApplicationQueryInfluenceContract {
    pub const fn forbid_all() -> Self {
        Self {
            permitted: BTreeSet::new(),
        }
    }

    pub fn permit(surfaces: impl IntoIterator<Item = ApplicationQueryObservableInfluence>) -> Self {
        Self {
            permitted: surfaces.into_iter().collect(),
        }
    }

    pub fn permit_all() -> Self {
        Self::permit(ApplicationQueryObservableInfluence::ALL)
    }

    pub fn permits(&self, surface: ApplicationQueryObservableInfluence) -> bool {
        self.permitted.contains(&surface)
    }

    pub fn permitted(
        &self,
    ) -> impl ExactSizeIterator<Item = ApplicationQueryObservableInfluence> + '_ {
        self.permitted.iter().copied()
    }
}
