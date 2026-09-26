//! Installation-authored semantic impact and adoption obligations.

use worth_query_declaration::facade::application_program::{
    ApplicationProgramMigrationAssessmentRequirement, ApplicationProgramRevision,
    ApplicationSemanticChangeKind, ApplicationSemanticDiff, ApplicationSemanticDiffDenial,
    ApplicationSemanticFamily,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationInvariantScopeTarget, ApplicationSchema, ApplicationSchemaBindingIdentity,
};

use super::scope_validation::installed_validation_scope;
use super::workflow_dependencies::{
    changed_workflow_dependencies, WorthQueryWorkflowDependencyName,
};
use crate::application_program::support::{
    WorthQueryProgramRuleKey, WorthQueryProgramSupportEntry, WorthQueryProgramSupportRoster,
};
use crate::facade::WorthQueryInstalledApplicationSchema;

const DEFAULT_COMPARISON_WORK_LIMIT: usize = 65_536;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramValidationScope {
    rule: WorthQueryProgramRuleKey,
    targets: Box<[ApplicationInvariantScopeTarget]>,
}

impl WorthQueryProgramValidationScope {
    pub fn rule(&self) -> &WorthQueryProgramRuleKey {
        &self.rule
    }

    pub fn targets(&self) -> &[ApplicationInvariantScopeTarget] {
        &self.targets
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramCustodyInventoryKind {
    OperationContinuation,
    ExternalEffectRecovery,
    ResourceCustody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramCustodyInventoryRequirement {
    kind: WorthQueryProgramCustodyInventoryKind,
    change: ApplicationSemanticChangeKind,
    subject: String,
}

impl WorthQueryProgramCustodyInventoryRequirement {
    pub const fn kind(&self) -> WorthQueryProgramCustodyInventoryKind {
        self.kind
    }

    pub const fn change(&self) -> ApplicationSemanticChangeKind {
        self.change
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

/// Owner-computed impact for one exact source/target pair and installation.
///
/// ```compile_fail,E0451
/// use worth_query_installation::facade::WorthQueryProgramAdoptionRequirements;
/// fn forged() -> WorthQueryProgramAdoptionRequirements {
///     WorthQueryProgramAdoptionRequirements {
///         schema_binding: unimplemented!(), source: unimplemented!(),
///         target: unimplemented!(), semantic_diff: unimplemented!(),
///         validation_scopes: unimplemented!(), added_rules: unimplemented!(),
///         migration_assessments: unimplemented!(), custody_inventory: unimplemented!(),
///         changed_workflow_dependencies: unimplemented!(),
///     }
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramAdoptionRequirements {
    schema_binding: ApplicationSchemaBindingIdentity,
    source: ApplicationProgramRevision,
    target: ApplicationProgramRevision,
    semantic_diff: ApplicationSemanticDiff,
    validation_scopes: Box<[WorthQueryProgramValidationScope]>,
    added_rules: Box<[WorthQueryProgramAddedRule]>,
    migration_assessments: Box<[ApplicationProgramMigrationAssessmentRequirement]>,
    custody_inventory: Box<[WorthQueryProgramCustodyInventoryRequirement]>,
    changed_workflow_dependencies: Box<[WorthQueryWorkflowDependencyName]>,
}

impl WorthQueryProgramAdoptionRequirements {
    pub fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }
    pub fn source(&self) -> &ApplicationProgramRevision {
        &self.source
    }
    pub fn target(&self) -> &ApplicationProgramRevision {
        &self.target
    }
    pub fn semantic_diff(&self) -> &ApplicationSemanticDiff {
        &self.semantic_diff
    }
    pub fn validation_scopes(&self) -> &[WorthQueryProgramValidationScope] {
        &self.validation_scopes
    }
    pub fn added_rules(&self) -> &[WorthQueryProgramAddedRule] {
        &self.added_rules
    }
    pub fn migration_assessment_requirements(
        &self,
    ) -> &[ApplicationProgramMigrationAssessmentRequirement] {
        &self.migration_assessments
    }
    pub fn custody_inventory_requirements(
        &self,
    ) -> &[WorthQueryProgramCustodyInventoryRequirement] {
        &self.custody_inventory
    }
    /// Binding identities and operation names the target no longer supplies
    /// with the source's meaning. A workflow definition whose node names any of
    /// them cannot be carried onto the target.
    pub fn changed_workflow_dependencies(&self) -> &[WorthQueryWorkflowDependencyName] {
        &self.changed_workflow_dependencies
    }
    /// The first name this retained workflow node acts through that the target
    /// stops supplying with the source's meaning.
    pub fn changed_workflow_node_dependency(
        &self,
        node: &crate::application_program::WorthQueryWorkflowNodeDependency,
    ) -> Option<&WorthQueryWorkflowDependencyName> {
        node.program_names().find_map(|name| {
            self.changed_workflow_dependencies
                .binary_search_by(|changed| changed.as_str().cmp(name))
                .ok()
                .map(|index| &self.changed_workflow_dependencies[index])
        })
    }
    pub fn requires_existing_state_validation(&self) -> bool {
        !self.validation_scopes.is_empty()
    }
    pub fn requires_migration_assessment(&self) -> bool {
        !self.migration_assessments.is_empty()
    }
    pub fn requires_custody_inventory(&self) -> bool {
        !self.custody_inventory.is_empty()
    }
    /// Positive authored equivalence. This does not claim that changed target
    /// rules have accepted the selected branch's live state.
    pub fn semantically_equivalent(&self) -> bool {
        self.semantic_diff.changes().is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramAdoptionRequirementsDenial {
    UnrosteredSource {
        revision: ApplicationProgramRevision,
    },
    UnrosteredTarget {
        revision: ApplicationProgramRevision,
    },
    ForeignSchemaBinding {
        roster: ApplicationSchemaBindingIdentity,
        installed: ApplicationSchemaBindingIdentity,
    },
    UnresolvedRuleScope {
        rule: WorthQueryProgramRuleKey,
    },
    SemanticComparison(ApplicationSemanticDiffDenial),
}

impl<Schema> WorthQueryProgramSupportRoster<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn adoption_requirements(
        &self,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        source: &ApplicationProgramRevision,
        target: &ApplicationProgramRevision,
    ) -> Result<WorthQueryProgramAdoptionRequirements, WorthQueryProgramAdoptionRequirementsDenial>
    {
        self.adoption_requirements_with_maximum_work(
            installed_schema,
            source,
            target,
            DEFAULT_COMPARISON_WORK_LIMIT,
        )
    }

    pub fn adoption_requirements_with_maximum_work(
        &self,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        source: &ApplicationProgramRevision,
        target: &ApplicationProgramRevision,
        maximum_comparison_work: usize,
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
        compile_requirements(
            installed_schema,
            installed,
            source_entry,
            target_entry,
            maximum_comparison_work,
        )
    }
}

fn compile_requirements<Schema: ApplicationSchema>(
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
    schema_binding: ApplicationSchemaBindingIdentity,
    source: &WorthQueryProgramSupportEntry,
    target: &WorthQueryProgramSupportEntry,
    maximum_comparison_work: usize,
) -> Result<WorthQueryProgramAdoptionRequirements, WorthQueryProgramAdoptionRequirementsDenial> {
    let semantic_diff = ApplicationSemanticDiff::compare(
        source.semantic_description(),
        target.semantic_description(),
        maximum_comparison_work,
    )
    .map_err(WorthQueryProgramAdoptionRequirementsDenial::SemanticComparison)?;
    let (validation_scopes, added_rules) =
        resolve_target_rule_scopes(installed_schema, source, target)?;
    let migration_assessments = semantic_diff
        .changes()
        .iter()
        .filter_map(|change| change.migration_assessment_requirement())
        .collect::<Vec<_>>();
    let custody_inventory = compile_custody_inventory(source, &semantic_diff);
    let changed_workflow_dependencies = changed_workflow_dependencies(source, &semantic_diff);
    Ok(WorthQueryProgramAdoptionRequirements {
        schema_binding,
        source: source.revision().clone(),
        target: target.revision().clone(),
        semantic_diff,
        validation_scopes: validation_scopes.into_boxed_slice(),
        added_rules: added_rules.into_boxed_slice(),
        migration_assessments: migration_assessments.into_boxed_slice(),
        custody_inventory: custody_inventory.into_boxed_slice(),
        changed_workflow_dependencies: changed_workflow_dependencies.into_boxed_slice(),
    })
}

fn resolve_target_rule_scopes<Schema: ApplicationSchema>(
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
    source: &WorthQueryProgramSupportEntry,
    target: &WorthQueryProgramSupportEntry,
) -> Result<
    (
        Vec<WorthQueryProgramValidationScope>,
        Vec<WorthQueryProgramAddedRule>,
    ),
    WorthQueryProgramAdoptionRequirementsDenial,
> {
    let mut scopes = Vec::new();
    let mut added = Vec::new();
    for rule in target.rules() {
        if source.declares_rule(rule) {
            continue;
        }
        let targets = installed_validation_scope(installed_schema, rule).ok_or_else(|| {
            WorthQueryProgramAdoptionRequirementsDenial::UnresolvedRuleScope { rule: rule.clone() }
        })?;
        scopes.push(WorthQueryProgramValidationScope {
            rule: rule.clone(),
            targets: targets.clone().into_boxed_slice(),
        });
        added.push(WorthQueryProgramAddedRule {
            rule: rule.clone(),
            validation_scope: targets.into_boxed_slice(),
        });
    }
    Ok((scopes, added))
}

fn compile_custody_inventory(
    source: &WorthQueryProgramSupportEntry,
    diff: &ApplicationSemanticDiff,
) -> Vec<WorthQueryProgramCustodyInventoryRequirement> {
    let mut requirements = Vec::new();
    for change in diff.changes() {
        let removed_or_changed = matches!(
            change.kind(),
            ApplicationSemanticChangeKind::Removed | ApplicationSemanticChangeKind::Changed
        );
        if change.family() == ApplicationSemanticFamily::Operations && removed_or_changed {
            requirements.push(custody_inventory_requirement(
                WorthQueryProgramCustodyInventoryKind::OperationContinuation,
                change.kind(),
                change.subject(),
            ));
            if source
                .effectful_action_subjects()
                .iter()
                .any(|subject| subject == change.subject())
            {
                requirements.push(custody_inventory_requirement(
                    WorthQueryProgramCustodyInventoryKind::ExternalEffectRecovery,
                    change.kind(),
                    change.subject(),
                ));
            }
        }
        if change.family() == ApplicationSemanticFamily::Resources && removed_or_changed {
            requirements.push(custody_inventory_requirement(
                WorthQueryProgramCustodyInventoryKind::ResourceCustody,
                change.kind(),
                change.subject(),
            ));
        }
    }
    requirements
}

fn custody_inventory_requirement(
    kind: WorthQueryProgramCustodyInventoryKind,
    change: ApplicationSemanticChangeKind,
    subject: &str,
) -> WorthQueryProgramCustodyInventoryRequirement {
    WorthQueryProgramCustodyInventoryRequirement {
        kind,
        change,
        subject: subject.to_owned(),
    }
}
