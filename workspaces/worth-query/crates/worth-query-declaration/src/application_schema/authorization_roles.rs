//! Closed vocabularies for roles and abilities, and the matrix between them.

/// A closed, code-identified value vocabulary, as `#[derive(WorthQueryVocabulary)]`
/// declares it. Codes are the durable wire values; variant order is declaration order.
pub trait ApplicationValueVocabulary: Copy + Eq + 'static {
    const VARIANTS: &'static [Self];

    fn code(self) -> u64;

    fn from_code(code: u64) -> Option<Self>;
}

/// The abilities of one scope entity, one marker per variant, as
/// `#[derive(WorthQueryAbilities)]` declares them.
pub trait ApplicationAbilityVocabulary<Schema>: Copy + Eq + 'static {
    /// The scope entity every ability of the vocabulary is checked on.
    type Scope;

    const ALL: &'static [Self];

    /// The ability's installed identifier, equal to its marker's reference name.
    fn ability_name(self) -> &'static str;

    /// The scope entity identifier the ability markers declare.
    fn scope_name() -> &'static str;
}

/// Which abilities each role grants. `role_abilities` inverts it: one rule per
/// ability over exactly the roles that grant it.
pub trait ApplicationRoleMatrix<Ability>: ApplicationValueVocabulary {
    fn abilities(self) -> &'static [Ability];
}
