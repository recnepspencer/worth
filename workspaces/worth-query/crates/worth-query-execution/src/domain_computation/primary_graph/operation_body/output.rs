//! Install fixed authored roles, then use the existing candidate correspondence owner.
use super::super::application_attempt::OutputRoleUse;
use super::super::{
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use crate::equality_type_id::EqualityTypeId;
use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationOutputContract},
    application_schema::{
        ApplicationOperationBodyOutputTarget, ApplicationOperationBodyProgram, ApplicationSchema,
    },
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

pub(super) struct InstalledBodyOutput {
    pub(super) target: ApplicationOperationBodyOutputTarget,
    pub(super) role: OutputRoleUse,
}

pub(super) fn install_outputs<
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    body: &ApplicationOperationBodyProgram,
) -> Result<Vec<InstalledBodyOutput>, Denial> {
    let denied = || Denial::new(Kind::OperationBodyUnavailable, Binding::IDENTITY);
    let mut outputs = Vec::new();
    for output in body.outputs() {
        if output.binding != Binding::IDENTITY
            || outputs
                .iter()
                .any(|existing: &InstalledBodyOutput| existing.role.name == output.role.name)
        {
            return Err(denied());
        }
        let role = Binding::Output::ROLES
            .iter()
            .find(|role| {
                role.name() == output.role.name
                    && role.entity() == output.role.entity
                    && role.posture() == output.role.posture
                    && role.cardinality() == output.role.cardinality
            })
            .ok_or_else(denied)?;
        use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as Posture;
        let target_matches = match &output.target {
            ApplicationOperationBodyOutputTarget::Scope => {
                role.posture() == Posture::Preserve && Some(role.entity()) == body.scope_entity()
            }
            ApplicationOperationBodyOutputTarget::Creation { key } => {
                role.posture() == Posture::Create
                    && body.creation().is_some_and(|creation| {
                        creation.key() == key && creation.entity() == role.entity()
                    })
            }
        };
        if !target_matches {
            return Err(denied());
        }
        let entity_type = schema
            .installed_entity_marker_type(role.entity())
            .ok_or_else(denied)?;
        outputs.push(InstalledBodyOutput {
            target: output.target.clone(),
            role: OutputRoleUse {
                name: role.name().to_owned(),
                entity_name: role.entity(),
                posture: role.posture(),
                cardinality: role.cardinality(),
                entity_type: entity_type.into(),
                contract_type: EqualityTypeId::of::<Binding::Output>(),
            },
        });
    }
    if Binding::Output::ROLES.iter().any(|role| {
        !role.cardinality().admits_absence()
            && !outputs.iter().any(|output| output.role.name == role.name())
    }) || Binding::Output::ROLE_FAMILIES
        .iter()
        .any(|family| family.minimum() > 0)
    {
        return Err(denied());
    }
    Ok(outputs)
}
