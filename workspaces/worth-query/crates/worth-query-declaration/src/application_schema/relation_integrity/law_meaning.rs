//! One declaration-owned representation of enabled relation-law meaning.
use super::super::ApplicationSchemaMember;
use super::ApplicationRelationIntegrity;

impl ApplicationRelationIntegrity {
    pub(crate) fn enabled_laws(self) -> impl Iterator<Item = (&'static str, u64)> {
        self.acyclic.then_some(("acyclic", 1)).into_iter()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ApplicationRelationLawMeaning {
    relation: String,
    from: String,
    to: String,
    laws: Vec<(&'static str, u64)>,
}

impl ApplicationRelationLawMeaning {
    pub(crate) fn from_members(members: &[ApplicationSchemaMember]) -> Vec<Self> {
        let mut meaning = members
            .iter()
            .filter_map(|member| {
                let ApplicationSchemaMember::Relation {
                    relation,
                    from,
                    to,
                    integrity,
                } = member
                else {
                    return None;
                };
                let laws: Vec<_> = integrity.enabled_laws().collect();
                (!laws.is_empty()).then(|| Self {
                    relation: relation.clone(),
                    from: from.clone(),
                    to: to.clone(),
                    laws,
                })
            })
            .collect::<Vec<_>>();
        meaning.sort_unstable();
        meaning
    }

    pub(crate) fn relation(&self) -> &str {
        &self.relation
    }

    pub(crate) fn canonical_fields(&self) -> impl Iterator<Item = (&'static str, String)> + '_ {
        [
            ("relation", self.relation.clone()),
            ("from", self.from.clone()),
            ("to", self.to.clone()),
        ]
        .into_iter()
        .chain(
            self.laws
                .iter()
                .map(|&(name, value)| (name, value.to_string())),
        )
    }
}
