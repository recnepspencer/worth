//! Resolving an added rule contract to the installed scope it reaches.

use worth_query_declaration::facade::application_schema::{
    ApplicationInvariantScopeTarget, ApplicationSchema,
};

use crate::application_program::support::WorthQueryProgramRuleKey;
use crate::facade::WorthQueryInstalledApplicationSchema;

/// The scope targets the installed catalog declares for one rule contract.
///
/// A rostered program may only declare rules this host installed, so a closed
/// roster cannot name a rule with no descriptor. Rather than trust that across
/// a crate boundary, an unresolvable rule is returned as a miss and refused by
/// the caller: a rule whose reach is unknown cannot be validated against, and a
/// host that cannot say what an adoption would check must not let it proceed.
pub(super) fn installed_validation_scope<Schema>(
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
    rule: &WorthQueryProgramRuleKey,
) -> Option<Vec<ApplicationInvariantScopeTarget>>
where
    Schema: ApplicationSchema,
{
    installed_schema
        .invariants()
        .descriptors()
        .find(|descriptor| {
            descriptor.identifier() == rule.identity()
                && descriptor.major() == rule.major()
                && descriptor.minor() == rule.minor()
                && descriptor.execution_point() == rule.execution_point()
        })
        .map(|descriptor| descriptor.applicability().to_vec())
}
