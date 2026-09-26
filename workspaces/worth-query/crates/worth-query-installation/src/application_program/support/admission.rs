use std::collections::BTreeSet;

use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramIdentity, ApplicationProgramRevision,
    ValidatedApplicationProgram,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaBindingIdentity, ApplicationSchemaMember,
};

use super::compatibility::{
    declared_rule_keys, first_rule_outside_catalog, first_unowned_installed_rule,
    installed_rule_keys, WorthQueryProgramRuleKey,
};
use super::{
    WorthQueryProgramActionDependency, WorthQueryProgramSupportEntry,
    WorthQueryProgramSupportRoster,
};
use crate::facade::WorthQueryInstalledApplicationSchema;

/// Refusal to support authored program meaning on one installed schema.
///
/// Every variant names the exact contract this host cannot honour, so a caller
/// learns what to install or re-author instead of retrying blind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramSupportDenial {
    /// A program that declares no feature governs nothing.
    EmptyProgram { program: ApplicationProgramIdentity },
    /// The program declares an invariant contract this host has not installed.
    UnsupportedRuleContract {
        program: ApplicationProgramIdentity,
        rule: WorthQueryProgramRuleKey,
    },
    /// The program acts through a binding or operation this host has not
    /// installed natively.
    UnsupportedAction {
        program: ApplicationProgramIdentity,
        binding: String,
    },
    /// An installed rule no rostered program declares would be enforced by a
    /// host where nothing understands it.
    UndeclaredInstalledRule { rule: WorthQueryProgramRuleKey },
    /// The same canonical revision was already admitted into this roster.
    DuplicateProgram {
        program: ApplicationProgramIdentity,
        revision: ApplicationProgramRevision,
    },
    /// The presented program was never admitted into this roster.
    UnrosteredProgram {
        program: ApplicationProgramIdentity,
        revision: ApplicationProgramRevision,
    },
    /// The roster was admitted against a different installed schema.
    ForeignSchemaBinding {
        rostered: ApplicationSchemaBindingIdentity,
        presented: ApplicationSchemaBindingIdentity,
    },
}

impl std::fmt::Display for WorthQueryProgramSupportDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyProgram { program } => {
                write!(
                    formatter,
                    "program declares no feature: {}",
                    program.as_str()
                )
            }
            Self::UnsupportedRuleContract { program, rule } => write!(
                formatter,
                "unsupported rule contract for {}: {rule}",
                program.as_str()
            ),
            Self::UnsupportedAction { program, binding } => write!(
                formatter,
                "unsupported program action for {}: {binding}",
                program.as_str()
            ),
            Self::UndeclaredInstalledRule { rule } => {
                write!(formatter, "undeclared installed rule: {rule}")
            }
            Self::DuplicateProgram { program, revision } => write!(
                formatter,
                "program {} already rostered at revision {revision}",
                program.as_str()
            ),
            Self::UnrosteredProgram { program, revision } => write!(
                formatter,
                "unrostered program {} at revision {revision}",
                program.as_str()
            ),
            Self::ForeignSchemaBinding {
                rostered,
                presented,
            } => write!(
                formatter,
                "foreign schema binding: roster {} presented {}",
                rostered.schema_identity().render_hex(),
                presented.schema_identity().render_hex()
            ),
        }
    }
}

impl std::error::Error for WorthQueryProgramSupportDenial {}

/// Admission of the programs one host supports on one installed schema.
///
/// Admission borrows the installed schema for its whole lifetime, so every
/// rostered program is compared against the same installed contracts. Closing
/// admission proves the roster owns every installed rule and yields the
/// immutable [`WorthQueryProgramSupportRoster`].
pub struct WorthQueryProgramSupportAdmission<'installation, Schema> {
    installed_schema: &'installation WorthQueryInstalledApplicationSchema<Schema>,
    installed_rules: BTreeSet<WorthQueryProgramRuleKey>,
    entries: Vec<WorthQueryProgramSupportEntry>,
}

impl<'installation, Schema> WorthQueryProgramSupportAdmission<'installation, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn for_installed_schema(
        installed_schema: &'installation WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Self {
        Self {
            installed_rules: installed_rule_keys(installed_schema),
            installed_schema,
            entries: Vec::new(),
        }
    }

    /// Admits one validated program as supported meaning on this installation.
    pub fn support<Program>(
        mut self,
        program: &ValidatedApplicationProgram<Schema, Program>,
    ) -> Result<Self, WorthQueryProgramSupportDenial>
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        let identity = program.identity();
        if self
            .entries
            .iter()
            .any(|entry| entry.revision() == program.revision())
        {
            return Err(WorthQueryProgramSupportDenial::DuplicateProgram {
                program: identity.clone(),
                revision: program.revision().clone(),
            });
        }
        require_governed_composition(program)?;
        let rules = self.require_supported_rules(program)?;
        self.require_supported_actions(program)?;
        self.entries.push(WorthQueryProgramSupportEntry::admitted(
            program.revision().clone(),
            identity.clone(),
            rules,
            program
                .actions()
                .iter()
                .map(|action| action.operation_type())
                .collect(),
            program
                .actions()
                .iter()
                .filter_map(|action| action.mutation_binding_type())
                .collect(),
            program.semantic_description().clone(),
            effectful_action_subjects(self.installed_schema, program),
            action_dependencies(self.installed_schema, program),
        ));
        Ok(self)
    }

    /// Closes admission once every installed rule has a declaring owner.
    pub fn close(
        self,
    ) -> Result<WorthQueryProgramSupportRoster<Schema>, WorthQueryProgramSupportDenial> {
        let rostered = self
            .entries
            .iter()
            .flat_map(WorthQueryProgramSupportEntry::rules)
            .cloned()
            .collect::<BTreeSet<_>>();
        if let Some(rule) = first_unowned_installed_rule(&self.installed_rules, &rostered) {
            return Err(WorthQueryProgramSupportDenial::UndeclaredInstalledRule {
                rule: rule.clone(),
            });
        }
        Ok(WorthQueryProgramSupportRoster::admitted(
            self.installed_schema.binding_identity(),
            self.entries,
        ))
    }

    fn require_supported_rules<Program>(
        &self,
        program: &ValidatedApplicationProgram<Schema, Program>,
    ) -> Result<BTreeSet<WorthQueryProgramRuleKey>, WorthQueryProgramSupportDenial> {
        let declared = declared_rule_keys(program.rules());
        match first_rule_outside_catalog(&declared, &self.installed_rules) {
            Some(rule) => Err(WorthQueryProgramSupportDenial::UnsupportedRuleContract {
                program: program.identity().clone(),
                rule: rule.clone(),
            }),
            None => Ok(declared),
        }
    }

    fn require_supported_actions<Program>(
        &self,
        program: &ValidatedApplicationProgram<Schema, Program>,
    ) -> Result<(), WorthQueryProgramSupportDenial> {
        for action in program.actions() {
            let installed = match action.mutation_binding_type() {
                Some(binding_type) => self
                    .installed_schema
                    .installed_mutation_binding_inventory()
                    .any(|binding| {
                        binding.binding_type() == binding_type
                            && binding.identity() == action.binding()
                    }),
                None => self
                    .installed_schema
                    .member_provenance
                    .admits_program_operation(
                        action.binding(),
                        action.operation_type(),
                        action.operation_input_type(),
                        action.operation_input_identity(),
                    ),
            };
            if !installed {
                return Err(WorthQueryProgramSupportDenial::UnsupportedAction {
                    program: program.identity().clone(),
                    binding: action.binding().to_owned(),
                });
            }
        }
        Ok(())
    }
}

fn effectful_action_subjects<Schema, Program>(
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
    program: &ValidatedApplicationProgram<Schema, Program>,
) -> BTreeSet<String>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    let external_operations = installed_schema
        .installed_declaration()
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::OperationExternalEffect { operation, .. } => {
                Some(operation.as_str())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    program
        .actions()
        .iter()
        .filter(|action| match action.mutation_binding_type() {
            Some(binding_type) => installed_schema
                .installed_mutation_binding_inventory()
                .find(|binding| {
                    binding.binding_type() == binding_type && binding.identity() == action.binding()
                })
                .is_some_and(|binding| external_operations.contains(binding.operation_name())),
            None => external_operations.contains(action.binding()),
        })
        .map(|action| action.semantic_subject())
        .collect()
}

fn action_dependencies<Schema, Program>(
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
    program: &ValidatedApplicationProgram<Schema, Program>,
) -> BTreeSet<WorthQueryProgramActionDependency>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    let mut dependencies = BTreeSet::new();
    for action in program.actions() {
        let subject = action.semantic_subject();
        dependencies.insert(WorthQueryProgramActionDependency {
            identity: action.binding().to_owned(),
            subject: subject.clone(),
        });
        let operation = action.mutation_binding_type().and_then(|binding_type| {
            installed_schema
                .installed_mutation_binding_inventory()
                .find(|binding| {
                    binding.binding_type() == binding_type && binding.identity() == action.binding()
                })
        });
        if let Some(binding) = operation {
            dependencies.insert(WorthQueryProgramActionDependency {
                identity: binding.operation_name().to_owned(),
                subject,
            });
        }
    }
    dependencies
}

fn require_governed_composition<Schema, Program>(
    program: &ValidatedApplicationProgram<Schema, Program>,
) -> Result<(), WorthQueryProgramSupportDenial> {
    if program.features().is_empty() {
        return Err(WorthQueryProgramSupportDenial::EmptyProgram {
            program: program.identity().clone(),
        });
    }
    Ok(())
}
