use std::collections::BTreeSet;

use worth_query_declaration::facade::application_program::ApplicationProgramRuleDeclaration;
use worth_query_declaration::facade::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationSchema,
};

use crate::facade::WorthQueryInstalledApplicationSchema;

/// One invariant contract named exactly as installation and a program must
/// agree on it: stable identity, declared version, and execution point.
///
/// Rule keys compare authored meaning. They are not installation authority and
/// carry no permission to evaluate the rule they name.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProgramRuleKey {
    identity: String,
    major: u16,
    minor: u16,
    execution_point: ApplicationInvariantExecutionPoint,
}

impl WorthQueryProgramRuleKey {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn major(&self) -> u16 {
        self.major
    }

    pub const fn minor(&self) -> u16 {
        self.minor
    }

    pub const fn execution_point(&self) -> ApplicationInvariantExecutionPoint {
        self.execution_point
    }
}

impl std::fmt::Display for WorthQueryProgramRuleKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} v{}.{} at {:?}",
            self.identity, self.major, self.minor, self.execution_point
        )
    }
}

/// The exact rule contracts one authored program declares.
pub(super) fn declared_rule_keys(
    rules: &[ApplicationProgramRuleDeclaration],
) -> BTreeSet<WorthQueryProgramRuleKey> {
    rules
        .iter()
        .map(|rule| WorthQueryProgramRuleKey {
            identity: rule.identity().to_owned(),
            major: rule.major(),
            minor: rule.minor(),
            execution_point: rule.execution_point(),
        })
        .collect()
}

/// The exact rule contracts one installed schema carries.
pub(super) fn installed_rule_keys<Schema>(
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> BTreeSet<WorthQueryProgramRuleKey>
where
    Schema: ApplicationSchema,
{
    installed_schema
        .invariants()
        .descriptors()
        .map(|descriptor| WorthQueryProgramRuleKey {
            identity: descriptor.identifier().to_owned(),
            major: descriptor.major(),
            minor: descriptor.minor(),
            execution_point: descriptor.execution_point(),
        })
        .collect()
}

/// Names the first rule a program declares that this host has not installed.
/// A program may never widen the installed invariant catalog.
pub(super) fn first_rule_outside_catalog<'declared>(
    declared: &'declared BTreeSet<WorthQueryProgramRuleKey>,
    installed: &BTreeSet<WorthQueryProgramRuleKey>,
) -> Option<&'declared WorthQueryProgramRuleKey> {
    first_absent(declared, installed)
}

/// Names the first installed rule no rostered program declares. Every
/// installed rule must have at least one owner, or the host would enforce
/// meaning nothing running here understands.
pub(super) fn first_unowned_installed_rule<'installed>(
    installed: &'installed BTreeSet<WorthQueryProgramRuleKey>,
    rostered: &BTreeSet<WorthQueryProgramRuleKey>,
) -> Option<&'installed WorthQueryProgramRuleKey> {
    first_absent(installed, rostered)
}

fn first_absent<'subject>(
    subject: &'subject BTreeSet<WorthQueryProgramRuleKey>,
    permitted: &BTreeSet<WorthQueryProgramRuleKey>,
) -> Option<&'subject WorthQueryProgramRuleKey> {
    subject.iter().find(|key| !permitted.contains(*key))
}
