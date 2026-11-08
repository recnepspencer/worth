//! Admit both installed meanings before their one shared semantic comparison.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryProgramSupportRoster<Schema> {
    /// Compare an authentic predecessor installation with this target roster.
    ///
    /// Each roster must belong to its supplied installed schema, and each
    /// revision must be a member of its respective roster. The schemas may have
    /// different Rust types. Newly governing rule scopes resolve against the
    /// target installation; prior dependencies come from the source entry.
    /// A retained rule key with a different installed descriptor refuses with
    /// `InstalledRuleContractChanged` before the unchanged-key shortcut applies.
    ///
    /// This returns descriptive requirements, never activation, execution or
    /// workflow-disposition authority. Live single-installation adoption uses
    /// the same path with one roster and schema in both roles.
    pub fn adoption_requirements_from_installed_predecessor<Before: ApplicationSchema>(
        &self,
        target_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        predecessor: &WorthQueryProgramSupportRoster<Before>,
        source_schema: &WorthQueryInstalledApplicationSchema<Before>,
        source: &ApplicationProgramRevision,
        target: &ApplicationProgramRevision,
        maximum_comparison_work: usize,
    ) -> Result<WorthQueryProgramAdoptionRequirements, WorthQueryProgramAdoptionRequirementsDenial>
    {
        let installed = target_schema.binding_identity();
        admit_binding(self.schema_binding(), &installed)?;
        admit_binding(
            predecessor.schema_binding(),
            &source_schema.binding_identity(),
        )?;
        let source_entry = predecessor.entry(source).ok_or(
            WorthQueryProgramAdoptionRequirementsDenial::UnrosteredSource { revision: *source },
        )?;
        let target_entry = self.entry(target).ok_or(
            WorthQueryProgramAdoptionRequirementsDenial::UnrosteredTarget { revision: *target },
        )?;
        admit_retained_rule_contracts(
            source_schema.invariants(),
            target_schema.invariants(),
            source_entry,
            target_entry,
        )?;
        compile_requirements(
            target_schema,
            installed,
            source_entry,
            target_entry,
            maximum_comparison_work,
        )
    }
}

fn admit_retained_rule_contracts(
    source_catalog: &crate::facade::WorthQueryInstalledApplicationInvariantCatalog,
    target_catalog: &crate::facade::WorthQueryInstalledApplicationInvariantCatalog,
    source: &WorthQueryProgramSupportEntry,
    target: &WorthQueryProgramSupportEntry,
) -> Result<(), WorthQueryProgramAdoptionRequirementsDenial> {
    use super::super::scope_validation::installed_rule_descriptor;
    if std::ptr::eq(source_catalog, target_catalog) {
        return Ok(());
    }
    // A shared key meant one contract in the old single-catalog entrance. Do
    // not carry that assumption across independently admitted installations.
    for rule in target
        .rules()
        .iter()
        .filter(|rule| source.declares_rule(rule))
    {
        let unresolved = || WorthQueryProgramAdoptionRequirementsDenial::UnresolvedRuleScope {
            rule: rule.clone(),
        };
        let before = installed_rule_descriptor(source_catalog, rule).ok_or_else(unresolved)?;
        let after = installed_rule_descriptor(target_catalog, rule).ok_or_else(unresolved)?;
        if before != after {
            return Err(
                WorthQueryProgramAdoptionRequirementsDenial::InstalledRuleContractChanged {
                    rule: rule.clone(),
                },
            );
        }
    }
    Ok(())
}

fn admit_binding(
    roster: &ApplicationSchemaBindingIdentity,
    installed: &ApplicationSchemaBindingIdentity,
) -> Result<(), WorthQueryProgramAdoptionRequirementsDenial> {
    if roster != installed {
        return Err(
            WorthQueryProgramAdoptionRequirementsDenial::ForeignSchemaBinding {
                roster: roster.clone(),
                installed: installed.clone(),
            },
        );
    }
    Ok(())
}
