//! The owner-compiled statement of what a branch must satisfy to adopt a
//! program.

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_declaration::facade::application_schema::{
    ApplicationInvariantScopeTarget, ApplicationSchema, ApplicationSchemaBindingIdentity,
};

use super::scope_validation::installed_validation_scope;
use crate::application_program::support::{
    WorthQueryProgramRuleKey, WorthQueryProgramSupportRoster,
};
use crate::facade::WorthQueryInstalledApplicationSchema;

/// One rule contract that begins governing when a branch moves from the source
/// program to the target program.
///
/// The scope is the installed catalog's own applicability for that contract,
/// never the caller's idea of it: it names what existing state the rule reaches
/// and therefore what an adoption has to put in front of it before the branch
/// can be said to satisfy the target program.
///
/// ```
/// use worth_query_installation::facade::{
///     WorthQueryProgramAddedRule, WorthQueryProgramAdoptionRequirements,
/// };
///
/// fn begins_governing(
///     requirements: &WorthQueryProgramAdoptionRequirements,
/// ) -> &[WorthQueryProgramAddedRule] {
///     requirements.added_rules()
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramAddedRule {
    rule: WorthQueryProgramRuleKey,
    validation_scope: Box<[ApplicationInvariantScopeTarget]>,
}

impl WorthQueryProgramAddedRule {
    pub fn rule(&self) -> &WorthQueryProgramRuleKey {
        &self.rule
    }

    pub fn validation_scope(&self) -> &[ApplicationInvariantScopeTarget] {
        &self.validation_scope
    }
}

/// What a branch must satisfy to move from one rostered program to another.
///
/// Requirements are evidence, never a permit. They describe what the host
/// computed for one (source, target) pair on one installed schema, so a caller
/// can see the cost and scope of an adoption before asking for it. The
/// operation that performs the adoption recomputes them from the branch's own
/// activation and compares; a value presented by a caller opens nothing that
/// the recomputation would not have opened by itself.
///
/// There is no public constructor. The only way to hold one is to have asked a
/// closed support roster for it.
///
/// ```compile_fail,E0451
/// use worth_query_installation::facade::WorthQueryProgramAdoptionRequirements;
///
/// fn forged() -> WorthQueryProgramAdoptionRequirements {
///     WorthQueryProgramAdoptionRequirements {
///         schema_binding: unimplemented!(),
///         source: unimplemented!(),
///         target: unimplemented!(),
///         added_rules: unimplemented!(),
///     }
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramAdoptionRequirements {
    schema_binding: ApplicationSchemaBindingIdentity,
    source: ApplicationProgramRevision,
    target: ApplicationProgramRevision,
    added_rules: Box<[WorthQueryProgramAddedRule]>,
}

impl WorthQueryProgramAdoptionRequirements {
    /// The installed schema these requirements were compiled against. An
    /// adoption judged on one installation says nothing about another.
    pub fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }

    pub fn source(&self) -> &ApplicationProgramRevision {
        &self.source
    }

    pub fn target(&self) -> &ApplicationProgramRevision {
        &self.target
    }

    /// The rules that start governing, in the roster's canonical rule order.
    pub fn added_rules(&self) -> &[WorthQueryProgramAddedRule] {
        &self.added_rules
    }

    /// Whether existing state has to be put in front of a rule before the
    /// target program can be said to hold on this branch.
    ///
    /// A rule the source declares and the target drops cannot be violated by
    /// state the branch is already carrying — it stops speaking — and a rule
    /// both declare has been governing that state all along. Only a rule that
    /// begins governing can find the branch already in breach.
    pub fn requires_existing_state_validation(&self) -> bool {
        !self.added_rules.is_empty()
    }
}

/// Why a host could not say what one adoption would demand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramAdoptionRequirementsDenial {
    /// The branch's current program is not on this roster, so what it declares
    /// — and therefore what the move adds — cannot be established.
    UnrosteredSource {
        revision: ApplicationProgramRevision,
    },
    /// The proposed program is not supported by this host.
    UnrosteredTarget {
        revision: ApplicationProgramRevision,
    },
    /// The roster was admitted against a different installed schema than the
    /// one asked to resolve its rules.
    ForeignSchemaBinding {
        roster: ApplicationSchemaBindingIdentity,
        installed: ApplicationSchemaBindingIdentity,
    },
    /// A rostered program declares a rule this installed catalog cannot
    /// resolve, so the scope the adoption would validate is unknown.
    ///
    /// This is defense in depth rather than an ordinarily constructible
    /// posture: support admission and scope resolution use the same rule key,
    /// while `ForeignSchemaBinding` ensures the roster and installed catalog
    /// belong to one schema binding. Keeping the denial explicit makes a
    /// future weakening of either boundary fail closed instead of silently
    /// treating an unknown scope as empty.
    UnresolvedRuleScope { rule: WorthQueryProgramRuleKey },
}

impl<Schema> WorthQueryProgramSupportRoster<Schema>
where
    Schema: ApplicationSchema,
{
    /// Compiles what a branch running `source` must satisfy to run `target`.
    ///
    /// `source == target` is not a denial: it compiles to requirements that add
    /// no rule. Whether re-adopting the program a branch already runs is a
    /// no-op or a mistake is the operation's judgement, not the catalog's.
    pub fn adoption_requirements(
        &self,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        source: &ApplicationProgramRevision,
        target: &ApplicationProgramRevision,
    ) -> Result<WorthQueryProgramAdoptionRequirements, WorthQueryProgramAdoptionRequirementsDenial>
    {
        let installed = installed_schema.binding_identity();
        if self.schema_binding() != &installed {
            return Err(
                WorthQueryProgramAdoptionRequirementsDenial::ForeignSchemaBinding {
                    roster: self.schema_binding().clone(),
                    installed,
                },
            );
        }
        let source_entry = self.entry(source).ok_or_else(|| {
            WorthQueryProgramAdoptionRequirementsDenial::UnrosteredSource {
                revision: source.clone(),
            }
        })?;
        let target_entry = self.entry(target).ok_or_else(|| {
            WorthQueryProgramAdoptionRequirementsDenial::UnrosteredTarget {
                revision: target.clone(),
            }
        })?;
        let mut added_rules = Vec::new();
        for rule in target_entry.rules() {
            if source_entry.declares_rule(rule) {
                continue;
            }
            let validation_scope =
                installed_validation_scope(installed_schema, rule).ok_or_else(|| {
                    WorthQueryProgramAdoptionRequirementsDenial::UnresolvedRuleScope {
                        rule: rule.clone(),
                    }
                })?;
            added_rules.push(WorthQueryProgramAddedRule {
                rule: rule.clone(),
                validation_scope: validation_scope.into_boxed_slice(),
            });
        }
        Ok(WorthQueryProgramAdoptionRequirements {
            schema_binding: installed,
            source: source.clone(),
            target: target.clone(),
            added_rules: added_rules.into_boxed_slice(),
        })
    }
}
