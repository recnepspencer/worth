use std::collections::{BTreeMap, BTreeSet};

use super::{
    ApplicationProgramRevision, ApplicationSemanticDescription, ApplicationSemanticFamily,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationSemanticChangeKind {
    Added,
    Removed,
    Changed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSemanticChange {
    family: ApplicationSemanticFamily,
    kind: ApplicationSemanticChangeKind,
    subject: String,
    source_meaning: Option<String>,
    target_meaning: Option<String>,
}

impl ApplicationSemanticChange {
    pub const fn family(&self) -> ApplicationSemanticFamily {
        self.family
    }

    pub const fn kind(&self) -> ApplicationSemanticChangeKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Canonical descriptive source meaning. This is inspection data, not an
    /// authority token or migration input.
    pub fn source_meaning(&self) -> Option<&str> {
        self.source_meaning.as_deref()
    }

    /// Canonical descriptive target meaning. This is inspection data, not an
    /// authority token or migration input.
    pub fn target_meaning(&self) -> Option<&str> {
        self.target_meaning.as_deref()
    }

    /// Names authored state-shape meaning whose compatibility must be assessed
    /// by the live owner. This does not claim that migration is necessary;
    /// only live target-rule/state evaluation can establish that.
    pub fn migration_assessment_requirement(
        &self,
    ) -> Option<ApplicationProgramMigrationAssessmentRequirement> {
        let requires_assessment = match (self.family, self.kind) {
            (ApplicationSemanticFamily::Features, ApplicationSemanticChangeKind::Changed)
            | (ApplicationSemanticFamily::Features, ApplicationSemanticChangeKind::Removed)
            | (ApplicationSemanticFamily::Ports, ApplicationSemanticChangeKind::Changed)
            | (ApplicationSemanticFamily::Ports, ApplicationSemanticChangeKind::Removed)
            | (ApplicationSemanticFamily::Connections, ApplicationSemanticChangeKind::Changed)
            | (ApplicationSemanticFamily::Connections, ApplicationSemanticChangeKind::Removed)
            | (ApplicationSemanticFamily::Outputs, ApplicationSemanticChangeKind::Changed)
            | (ApplicationSemanticFamily::Outputs, ApplicationSemanticChangeKind::Removed) => true,
            _ => false,
        };
        requires_assessment.then(|| ApplicationProgramMigrationAssessmentRequirement {
            family: self.family,
            subject: self.subject.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationProgramMigrationAssessmentRequirement {
    family: ApplicationSemanticFamily,
    subject: String,
}

impl ApplicationProgramMigrationAssessmentRequirement {
    pub const fn family(&self) -> ApplicationSemanticFamily {
        self.family
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSemanticDiff {
    source: ApplicationProgramRevision,
    target: ApplicationProgramRevision,
    changes: Box<[ApplicationSemanticChange]>,
    equivalent_families: Box<[ApplicationSemanticFamily]>,
    comparison_work_units: usize,
}

impl ApplicationSemanticDiff {
    pub fn source(&self) -> &ApplicationProgramRevision {
        &self.source
    }

    pub fn target(&self) -> &ApplicationProgramRevision {
        &self.target
    }

    pub fn changes(&self) -> &[ApplicationSemanticChange] {
        &self.changes
    }

    pub fn equivalent_families(&self) -> &[ApplicationSemanticFamily] {
        &self.equivalent_families
    }

    pub const fn comparison_work_units(&self) -> usize {
        self.comparison_work_units
    }

    pub fn compare(
        source: &ApplicationSemanticDescription,
        target: &ApplicationSemanticDescription,
        maximum_work_units: usize,
    ) -> Result<Self, ApplicationSemanticDiffDenial> {
        let comparison_work_units = source.facts().len().saturating_add(target.facts().len());
        if comparison_work_units > maximum_work_units {
            return Err(ApplicationSemanticDiffDenial::WorkLimitExceeded {
                maximum_work_units,
                consumed_work_units: comparison_work_units,
            });
        }
        let source_facts = index(source);
        let target_facts = index(target);
        let keys = source_facts
            .keys()
            .chain(target_facts.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut changes = Vec::new();
        for (family, subject) in keys {
            let key = (family, subject.clone());
            let change = match (source_facts.get(&key), target_facts.get(&key)) {
                (None, Some(target)) => Some((
                    ApplicationSemanticChangeKind::Added,
                    None,
                    Some((*target).to_owned()),
                )),
                (Some(source), None) => Some((
                    ApplicationSemanticChangeKind::Removed,
                    Some((*source).to_owned()),
                    None,
                )),
                (Some(source), Some(target)) if source != target => Some((
                    ApplicationSemanticChangeKind::Changed,
                    Some((*source).to_owned()),
                    Some((*target).to_owned()),
                )),
                _ => None,
            };
            if let Some((kind, source_meaning, target_meaning)) = change {
                changes.push(ApplicationSemanticChange {
                    family,
                    kind,
                    subject,
                    source_meaning,
                    target_meaning,
                });
            }
        }
        let changed_families = changes
            .iter()
            .map(ApplicationSemanticChange::family)
            .collect::<BTreeSet<_>>();
        let equivalent_families = ApplicationSemanticFamily::ALL
            .into_iter()
            .filter(|family| !changed_families.contains(family))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Ok(Self {
            source: source.revision().clone(),
            target: target.revision().clone(),
            changes: changes.into_boxed_slice(),
            equivalent_families,
            comparison_work_units,
        })
    }
}

fn index(
    description: &ApplicationSemanticDescription,
) -> BTreeMap<(ApplicationSemanticFamily, String), &str> {
    description
        .facts()
        .iter()
        .map(|fact| ((fact.family(), fact.subject().to_owned()), fact.meaning()))
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationSemanticDiffDenial {
    WorkLimitExceeded {
        maximum_work_units: usize,
        consumed_work_units: usize,
    },
}
