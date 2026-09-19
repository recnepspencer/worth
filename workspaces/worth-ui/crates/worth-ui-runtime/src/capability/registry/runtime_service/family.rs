#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiRuntimeServiceFamily {
    Portal,
    Focus,
    Motion,
    CommandRouting,
    Scroll,
    Selection,
}

impl UiRuntimeServiceFamily {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const ALL: [Self; 6] = [
        Self::Portal,
        Self::Focus,
        Self::Motion,
        Self::CommandRouting,
        Self::Scroll,
        Self::Selection,
    ];

    #[cfg(test)]
    pub(crate) const fn stable_name(self) -> &'static str {
        match self {
            Self::Portal => "portal",
            Self::Focus => "focus",
            Self::Motion => "motion",
            Self::CommandRouting => "command-routing",
            Self::Scroll => "scroll",
            Self::Selection => "selection",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Portal => 0,
            Self::Focus => 1,
            Self::Motion => 2,
            Self::CommandRouting => 3,
            Self::Scroll => 4,
            Self::Selection => 5,
        }
    }
}
