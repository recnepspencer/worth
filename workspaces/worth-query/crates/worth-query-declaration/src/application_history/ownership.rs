//! Installed ownership meaning; relation names never supply stored membership.
use super::ApplicationHistoryDefinitionDenial;
use crate::application_schema::ApplicationRelationRef;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationHistoryOwnershipRelation {
    pub relation: String,
    pub from: String,
    pub to: String,
}

impl ApplicationHistoryOwnershipRelation {
    pub(super) fn of<S, R, From, To>(relation: ApplicationRelationRef<S, R, From, To>) -> Self {
        Self {
            relation: relation.name().to_owned(),
            from: relation.from().to_owned(),
            to: relation.to().to_owned(),
        }
    }
}

pub(super) fn validate(
    relations: &[ApplicationHistoryOwnershipRelation],
) -> Result<(), ApplicationHistoryDefinitionDenial> {
    let mut seen = std::collections::BTreeSet::new();
    for relation in relations {
        if [&relation.relation, &relation.from, &relation.to]
            .into_iter()
            .any(|name| !super::definition::name_is_valid(name))
            || !seen.insert(&relation.relation)
        {
            return Err(ApplicationHistoryDefinitionDenial::InvalidOwnership);
        }
    }
    Ok(())
}

/// Type-level reachability is installation eligibility, never instance proof.
pub fn history_ownership_reaches(
    relations: &[ApplicationHistoryOwnershipRelation],
    root: &str,
    target: &str,
) -> bool {
    let mut reached = std::collections::BTreeSet::from([root]);
    loop {
        if reached.contains(target) {
            return true;
        }
        let before = reached.len();
        for relation in relations {
            if reached.contains(relation.from.as_str()) {
                reached.insert(relation.to.as_str());
            }
        }
        if reached.len() == before {
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn edge(name: &str, from: &str, to: &str) -> ApplicationHistoryOwnershipRelation {
        ApplicationHistoryOwnershipRelation {
            relation: name.into(),
            from: from.into(),
            to: to.into(),
        }
    }
    #[test]
    fn declared_type_reachability_terminates_at_cycles_and_keeps_unrelated_scopes_out() {
        let edges = [
            edge("Members", "Model", "Member"),
            edge("Children", "Member", "Child"),
            edge("Back", "Child", "Member"),
        ];
        assert!(history_ownership_reaches(&edges, "Model", "Model"));
        assert!(history_ownership_reaches(&edges, "Model", "Child"));
        assert!(!history_ownership_reaches(&edges, "Model", "Principal"));
        assert!(!history_ownership_reaches(&edges, "Child", "Model"));
    }
    #[test]
    fn ownership_meaning_rejects_duplicate_edges_and_invalid_identifiers() {
        let valid = edge("Members", "Model", "Member");
        assert_eq!(validate(std::slice::from_ref(&valid)), Ok(()));
        assert_eq!(
            validate(&[valid.clone(), valid]),
            Err(ApplicationHistoryDefinitionDenial::InvalidOwnership)
        );
        for invalid in [
            edge("", "Model", "Member"),
            edge("Members", "Model Root", "Member"),
            edge("Members", "Model", ""),
        ] {
            assert_eq!(
                validate(&[invalid]),
                Err(ApplicationHistoryDefinitionDenial::InvalidOwnership)
            );
        }
    }
}
