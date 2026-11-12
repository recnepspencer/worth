use worth_query_declaration::facade::application_schema::ApplicationMutationPreconditionTarget;

use super::WorthQueryInstalledAbilityRequirement;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledMutationPrecondition {
    target: ApplicationMutationPreconditionTarget,
}

impl WorthQueryInstalledMutationPrecondition {
    pub(crate) const fn new(target: ApplicationMutationPreconditionTarget) -> Self {
        Self { target }
    }

    pub const fn target(&self) -> &ApplicationMutationPreconditionTarget {
        &self.target
    }
}

pub(crate) fn compile_precondition_contract(
    mut targets: Vec<ApplicationMutationPreconditionTarget>,
    decision_reads: &crate::domain_operation::WorthQueryOperationGraphReadContract,
    abilities: &[WorthQueryInstalledAbilityRequirement],
) -> Result<Vec<WorthQueryInstalledMutationPrecondition>, ()> {
    targets.sort();
    if targets.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(());
    }
    targets
        .into_iter()
        .map(|target| {
            if !is_installed_decision_read(&target, decision_reads)
                || abilities
                    .iter()
                    .any(|ability| ability.scope_entity() != target.entity())
            {
                return Err(());
            }
            Ok(WorthQueryInstalledMutationPrecondition::new(target))
        })
        .collect()
}

fn is_installed_decision_read(
    target: &ApplicationMutationPreconditionTarget,
    decision_reads: &crate::domain_operation::WorthQueryOperationGraphReadContract,
) -> bool {
    let Some(field) = worth_foundational::facade::FieldKey::new(target.field_name()) else {
        return false;
    };
    decision_reads
        .roles()
        .iter()
        .flat_map(|role| role.read_scopes())
        .any(|read| {
            let crate::domain_operation::WorthQueryOperationGraphReadScope::NativeProjection(read) =
                read
            else {
                return false;
            };
            read.entity().semantic_key() == target.entity()
                && read.aspect().as_str() == target.aspect()
                && read.admits_field(&field)
        })
}
