//! Descriptive endpoint domains; observed member authority belongs to Native.

/// An owned endpoint namespace in a portable structural recipe.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationGraphMemberMeaning {
    Entity(String),
    Projected(String),
}

/// A schema-authored endpoint selector, never an observed record or member.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationGraphMemberSelector<'name> {
    Entity(&'name str),
    Projected(&'name str),
}

impl ApplicationGraphMemberMeaning {
    pub fn name(&self) -> &str {
        self.borrowed().name()
    }
    pub fn borrowed(&self) -> ApplicationGraphMemberSelector<'_> {
        match self {
            Self::Entity(name) => ApplicationGraphMemberSelector::Entity(name),
            Self::Projected(name) => ApplicationGraphMemberSelector::Projected(name),
        }
    }
    pub fn entity(&self) -> Option<&str> {
        match self {
            Self::Entity(name) => Some(name),
            Self::Projected(_) => None,
        }
    }
}
impl From<String> for ApplicationGraphMemberMeaning {
    fn from(name: String) -> Self {
        Self::Entity(name)
    }
}
impl From<&str> for ApplicationGraphMemberMeaning {
    fn from(name: &str) -> Self {
        Self::Entity(name.to_owned())
    }
}
impl<'name> ApplicationGraphMemberSelector<'name> {
    pub const fn name(self) -> &'name str {
        match self {
            Self::Entity(name) | Self::Projected(name) => name,
        }
    }
    pub fn into_meaning(self) -> ApplicationGraphMemberMeaning {
        match self {
            Self::Entity(name) => ApplicationGraphMemberMeaning::Entity(name.to_owned()),
            Self::Projected(name) => ApplicationGraphMemberMeaning::Projected(name.to_owned()),
        }
    }
}
