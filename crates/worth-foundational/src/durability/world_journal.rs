#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorldDurableJournal {
    Branches,
    Occurrences,
    Custody,
}
impl WorldDurableJournal {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Branches => "world.branches",
            Self::Occurrences => "world.occurrences",
            Self::Custody => "world.custody",
        }
    }
}
