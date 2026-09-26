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
        let requires_assessment = matches!(
            (self.family, self.kind),
            (
                ApplicationSemanticFamily::Features,
                ApplicationSemanticChangeKind::Changed
            ) | (
                ApplicationSemanticFamily::Features,
                ApplicationSemanticChangeKind::Removed
            ) | (
                ApplicationSemanticFamily::Ports,
                ApplicationSemanticChangeKind::Changed
            ) | (
                ApplicationSemanticFamily::Ports,
                ApplicationSemanticChangeKind::Removed
            ) | (
                ApplicationSemanticFamily::Connections,
                ApplicationSemanticChangeKind::Changed
            ) | (
                ApplicationSemanticFamily::Connections,
                ApplicationSemanticChangeKind::Removed
            ) | (
                ApplicationSemanticFamily::Outputs,
                ApplicationSemanticChangeKind::Changed
            ) | (
                ApplicationSemanticFamily::Outputs,
                ApplicationSemanticChangeKind::Removed
            )
        );
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
            let source_meanings = source_facts.get(&key).map_or(&[][..], Vec::as_slice);
            let target_meanings = target_facts.get(&key).map_or(&[][..], Vec::as_slice);
            append_subject_changes(
                &mut changes,
                family,
                &subject,
                source_meanings,
                target_meanings,
            );
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
) -> BTreeMap<(ApplicationSemanticFamily, String), Vec<&str>> {
    let mut facts = BTreeMap::<_, Vec<_>>::new();
    for fact in description.facts() {
        facts
            .entry((fact.family(), fact.subject().to_owned()))
            .or_default()
            .push(fact.canonical_meaning());
    }
    for meanings in facts.values_mut() {
        meanings.sort_unstable();
    }
    facts
}

fn append_subject_changes(
    changes: &mut Vec<ApplicationSemanticChange>,
    family: ApplicationSemanticFamily,
    subject: &str,
    source_meanings: &[&str],
    target_meanings: &[&str],
) {
    let mut unmatched_source = Vec::new();
    let mut unmatched_target = Vec::new();
    let (mut source_index, mut target_index) = (0, 0);
    while source_index < source_meanings.len() && target_index < target_meanings.len() {
        match source_meanings[source_index].cmp(target_meanings[target_index]) {
            std::cmp::Ordering::Equal => {
                source_index += 1;
                target_index += 1;
            }
            std::cmp::Ordering::Less => {
                unmatched_source.push(source_meanings[source_index]);
                source_index += 1;
            }
            std::cmp::Ordering::Greater => {
                unmatched_target.push(target_meanings[target_index]);
                target_index += 1;
            }
        }
    }
    unmatched_source.extend_from_slice(&source_meanings[source_index..]);
    unmatched_target.extend_from_slice(&target_meanings[target_index..]);

    let changed_count = unmatched_source.len().min(unmatched_target.len());
    for index in 0..changed_count {
        changes.push(change(
            family,
            ApplicationSemanticChangeKind::Changed,
            subject,
            Some(unmatched_source[index]),
            Some(unmatched_target[index]),
        ));
    }
    for source in &unmatched_source[changed_count..] {
        changes.push(change(
            family,
            ApplicationSemanticChangeKind::Removed,
            subject,
            Some(source),
            None,
        ));
    }
    for target in &unmatched_target[changed_count..] {
        changes.push(change(
            family,
            ApplicationSemanticChangeKind::Added,
            subject,
            None,
            Some(target),
        ));
    }
}

fn change(
    family: ApplicationSemanticFamily,
    kind: ApplicationSemanticChangeKind,
    subject: &str,
    source_meaning: Option<&str>,
    target_meaning: Option<&str>,
) -> ApplicationSemanticChange {
    ApplicationSemanticChange {
        family,
        kind,
        subject: subject.to_owned(),
        source_meaning: source_meaning.map(str::to_owned),
        target_meaning: target_meaning.map(str::to_owned),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationSemanticDiffDenial {
    WorkLimitExceeded {
        maximum_work_units: usize,
        consumed_work_units: usize,
    },
}
