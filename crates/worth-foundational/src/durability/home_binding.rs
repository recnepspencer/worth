use crate::CanonicalDigestId;

/// Recorded program meaning, never declaration or installation authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HomeProgramBinding {
    identity: String,
    revision: CanonicalDigestId,
}

impl HomeProgramBinding {
    pub fn new(identity: String, revision: CanonicalDigestId) -> Self {
        Self { identity, revision }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn revision(&self) -> CanonicalDigestId {
        self.revision
    }
}

/// Descriptive home binding. A caller may describe it, but only Query's
/// admitted binding carrier permits production home construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationHomeBinding {
    package: CanonicalDigestId,
    schema: CanonicalDigestId,
    selected: HomeProgramBinding,
    roster: Vec<HomeProgramBinding>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationHomeBindingDescriptionDenial {
    EmptyRoster,
    DuplicateRevision,
    SelectedProgramNotRostered,
}

impl ApplicationHomeBinding {
    pub fn new(
        package: CanonicalDigestId,
        schema: CanonicalDigestId,
        selected: HomeProgramBinding,
        mut roster: Vec<HomeProgramBinding>,
    ) -> Result<Self, ApplicationHomeBindingDescriptionDenial> {
        use ApplicationHomeBindingDescriptionDenial as Denial;
        if roster.is_empty() {
            return Err(Denial::EmptyRoster);
        }
        roster.sort_unstable_by_key(HomeProgramBinding::revision);
        if roster
            .windows(2)
            .any(|pair| pair[0].revision == pair[1].revision)
        {
            return Err(Denial::DuplicateRevision);
        }
        if !roster.contains(&selected) {
            return Err(Denial::SelectedProgramNotRostered);
        }
        Ok(Self {
            package,
            schema,
            selected,
            roster,
        })
    }

    pub const fn package(&self) -> CanonicalDigestId {
        self.package
    }

    pub const fn schema(&self) -> CanonicalDigestId {
        self.schema
    }

    pub fn selected(&self) -> &HomeProgramBinding {
        &self.selected
    }

    pub fn roster(&self) -> &[HomeProgramBinding] {
        &self.roster
    }

    pub fn program(&self, revision: CanonicalDigestId) -> Option<&HomeProgramBinding> {
        self.roster
            .binary_search_by_key(&revision, HomeProgramBinding::revision)
            .ok()
            .map(|index| &self.roster[index])
    }
}
