//! One declaration-owned representation of enabled relation-law meaning.
use super::super::ApplicationSchemaMember;
use super::ApplicationRelationIntegrity;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum ApplicationRelationLawValue {
    Number(u64),
    KindSet(Vec<&'static str>),
}
impl ApplicationRelationLawValue {
    pub(crate) fn canonical_text(&self) -> String {
        match self {
            Self::Number(value) => value.to_string(),
            Self::KindSet(kinds) => {
                serde_json::to_string(kinds).expect("entity kind names serialize")
            }
        }
    }
}

impl ApplicationRelationIntegrity {
    pub(crate) fn enabled_laws(
        self,
    ) -> impl Iterator<Item = (&'static str, ApplicationRelationLawValue)> {
        [
            self.acyclic.then_some(("acyclic", 1)),
            self.symmetric.then_some(("symmetric", 1)),
            self.unique.map(|scope| ("unique", scope.canonical_value())),
            self.same_partition_endpoints
                .then_some(("same_partition_endpoints", 1)),
        ]
        .into_iter()
        .flatten()
        .map(|(name, value)| (name, ApplicationRelationLawValue::Number(value)))
        .chain(self.reaches_at_least.into_iter().flat_map(|minimum| {
            [
                (
                    "reaches_at_least",
                    ApplicationRelationLawValue::Number(u64::from(
                        minimum.minimum_reachable_targets,
                    )),
                ),
                (
                    "reaches_at_least.kinds",
                    ApplicationRelationLawValue::KindSet(minimum.normalized_kinds()),
                ),
            ]
        }))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ApplicationRelationLawMeaning {
    relation: String,
    from: String,
    to: String,
    laws: Vec<(&'static str, ApplicationRelationLawValue)>,
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
                .map(|(name, value)| (*name, value.canonical_text())),
        )
    }
}
