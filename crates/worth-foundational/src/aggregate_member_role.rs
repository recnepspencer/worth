/// Role a member plays for its group's declared continuity rule.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum AggregateMemberRole {
    Interior,
    /// Bears the scheme's declared anchor (or producer) role.
    Anchor,
    /// Encloses the group under boundary continuity.
    Boundary,
}
