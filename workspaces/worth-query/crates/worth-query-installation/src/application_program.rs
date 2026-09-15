use std::marker::PhantomData;

use worth_query_declaration::facade::application_program::{
    ApplicationConnectionDeclaration, ApplicationFeatureDeclaration, ApplicationFeaturePosture,
    ApplicationProgramDefinition, ApplicationProgramIdentity,
    ApplicationProgramInventoryDeclaration, ApplicationProgramInventoryIdentity,
    ApplicationProgramRuleDeclaration, ValidatedApplicationProgram,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
};

/// Failure to bind validated program meaning to its exact installed schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationProgramInstallationDenial {
    kind: WorthQueryApplicationProgramInstallationDenialKind,
    subject: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationProgramInstallationDenialKind {
    MissingInstalledRule,
    ConnectionIdentityMismatch,
    IncompleteExecutionBindings,
    RootDemandMismatch,
}

impl WorthQueryApplicationProgramInstallationDenial {
    pub const fn kind(&self) -> WorthQueryApplicationProgramInstallationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn new(
        kind: WorthQueryApplicationProgramInstallationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }
}

impl std::fmt::Display for WorthQueryApplicationProgramInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application program installation denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryApplicationProgramInstallationDenial {}

fn require_installed_rule(
    identity: &str,
    installed: bool,
) -> Result<(), WorthQueryApplicationProgramInstallationDenial> {
    if !installed {
        return Err(WorthQueryApplicationProgramInstallationDenial {
            kind: WorthQueryApplicationProgramInstallationDenialKind::MissingInstalledRule,
            subject: identity.to_owned(),
        });
    }
    Ok(())
}

/// Installed program meaning affine to one schema installation.
pub struct WorthQueryInstalledApplicationProgram<Schema, Program> {
    identity: ApplicationProgramIdentity,
    schema_binding: ApplicationSchemaBindingIdentity,
    features: Box<[ApplicationFeatureDeclaration]>,
    connections: Box<[ApplicationConnectionDeclaration]>,
    rules: Box<[ApplicationProgramRuleDeclaration]>,
    inventories: Box<[ApplicationProgramInventoryDeclaration]>,
    marker: PhantomData<fn() -> (Schema, Program)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledProgramInventoryPosture {
    Available,
    Unavailable { feature: String },
    Missing,
}

impl<Schema, Program> WorthQueryInstalledApplicationProgram<Schema, Program> {
    pub fn identity(&self) -> &ApplicationProgramIdentity {
        &self.identity
    }
    pub fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }
    pub fn features(&self) -> &[ApplicationFeatureDeclaration] {
        &self.features
    }
    pub fn connections(&self) -> &[ApplicationConnectionDeclaration] {
        &self.connections
    }

    pub fn rules(&self) -> &[ApplicationProgramRuleDeclaration] {
        &self.rules
    }

    pub fn contains_connection_type<Connection: 'static>(&self) -> bool {
        let node_type = std::any::TypeId::of::<Connection>();
        self.connections
            .iter()
            .any(|connection| connection.node_type() == node_type)
    }

    pub fn inventories(&self) -> &[ApplicationProgramInventoryDeclaration] {
        &self.inventories
    }

    pub fn inventory<Inventory>(&self) -> Option<&ApplicationProgramInventoryDeclaration>
    where
        Inventory: ApplicationProgramInventoryIdentity,
    {
        self.inventories
            .iter()
            .find(|candidate| candidate.marker_type() == std::any::TypeId::of::<Inventory>())
    }

    pub fn inventory_feature_closure<Inventory>(
        &self,
    ) -> Option<std::collections::BTreeSet<std::any::TypeId>>
    where
        Inventory: ApplicationProgramInventoryIdentity,
    {
        let inventory = self.inventory::<Inventory>()?;
        let mut closure = inventory
            .outputs()
            .iter()
            .map(|output| output.feature_type())
            .collect::<std::collections::BTreeSet<_>>();
        loop {
            let before = closure.len();
            for connection in &self.connections {
                if closure.contains(&connection.target_feature_type()) {
                    closure.insert(connection.source_feature_type());
                }
            }
            if closure.len() == before {
                break;
            }
        }
        Some(closure)
    }

    pub fn inventory_posture<Inventory>(&self) -> WorthQueryInstalledProgramInventoryPosture
    where
        Inventory: ApplicationProgramInventoryIdentity,
    {
        let Some(closure) = self.inventory_feature_closure::<Inventory>() else {
            return WorthQueryInstalledProgramInventoryPosture::Missing;
        };
        if let Some(feature) = self.features.iter().find(|feature| {
            closure.contains(&feature.type_id())
                && feature.posture() == ApplicationFeaturePosture::Unavailable
        }) {
            WorthQueryInstalledProgramInventoryPosture::Unavailable {
                feature: feature.identity().to_owned(),
            }
        } else {
            WorthQueryInstalledProgramInventoryPosture::Available
        }
    }
    pub fn inventory_accepts_root<Inventory, Root>(&self) -> bool
    where
        Inventory: ApplicationProgramInventoryIdentity,
        Root: 'static,
    {
        let Some(closure) = self.inventory_feature_closure::<Inventory>() else {
            return false;
        };
        let Some(root) = self
            .connections
            .iter()
            .find(|connection| connection.node_type() == std::any::TypeId::of::<Root>())
        else {
            return false;
        };
        closure.contains(&root.target_feature_type())
    }
}

pub fn install_application_program<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    installed_schema: &crate::facade::WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<
    WorthQueryInstalledApplicationProgram<Schema, Program>,
    WorthQueryApplicationProgramInstallationDenial,
>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    for rule in program.rules() {
        let installed = installed_schema
            .invariants()
            .descriptors()
            .any(|candidate| {
                candidate.identifier() == rule.identity()
                    && candidate.major() == rule.major()
                    && candidate.minor() == rule.minor()
                    && candidate.execution_point() == rule.execution_point()
            });
        require_installed_rule(rule.identity(), installed)?;
    }
    Ok(WorthQueryInstalledApplicationProgram {
        identity: program.identity().clone(),
        schema_binding: installed_schema.binding_identity(),
        features: program.features().to_vec().into_boxed_slice(),
        connections: program.connections().to_vec().into_boxed_slice(),
        rules: program.rules().to_vec().into_boxed_slice(),
        inventories: program.inventories().to_vec().into_boxed_slice(),
        marker: PhantomData,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_declared_rule_must_exist_in_the_installed_schema() {
        let denial = require_installed_rule("missing-rule", false)
            .expect_err("program installation cannot invent an invariant rule");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationProgramInstallationDenialKind::MissingInstalledRule
        );
        assert_eq!(denial.subject(), "missing-rule");
    }
}
