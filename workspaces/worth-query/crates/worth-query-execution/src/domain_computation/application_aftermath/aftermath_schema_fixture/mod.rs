//! Minimal application schema that installs aftermath through the production door.

mod declaration;
mod operations;

use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
    ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledAftermathContract,
    WorthQueryInstalledPackageIndex, WorthQueryOperationGraphReadScope,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

use declaration::{
    AftermathFixtureSchema, ApproveEmergencyAccess, AuditRetention, Charge, FixtureInput,
    FreezeAccount, FreezeAccountFields, FreezeBalance, FreezeNote, LegalHold, NotifyDeath,
    ReleaseEstate, Transfer, TransferLarge, TransferSmall, WireTransferFinal,
};

fn op<Operation>() -> ApplicationOperationRef<AftermathFixtureSchema, Operation, FixtureInput>
where
    Operation: ApplicationOperationMarkerIdentity<AftermathFixtureSchema> + 'static,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = FixtureInput>,
{
    ApplicationOperationRef::from_declaration()
}

fn installed_schema(
) -> worth_query_installation::facade::WorthQueryInstalledApplicationSchema<AftermathFixtureSchema>
{
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "aftermath-fixture",
        1,
        0,
    ))
    .application_schema(AftermathFixtureSchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap();
    index
        .bind_application_schema(AftermathFixtureSchema::declaration().unwrap())
        .unwrap()
}

fn aftermath_of<Operation>() -> WorthQueryInstalledAftermathContract
where
    Operation: ApplicationOperationMarkerIdentity<AftermathFixtureSchema> + 'static,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = FixtureInput>,
{
    installed_schema()
        .installed_operation(op::<Operation>())
        .expect("operation installs")
        .contracts()
        .aftermath()
        .expect("aftermath declared")
        .clone()
}

pub(crate) fn release_estate() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<ReleaseEstate>()
}
pub(crate) fn transfer() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<Transfer>()
}
pub(crate) fn transfer_small() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<TransferSmall>()
}
pub(crate) fn transfer_large() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<TransferLarge>()
}
pub(crate) fn freeze_account() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<FreezeAccount>()
}
pub(crate) fn freeze_account_fields() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<FreezeAccountFields>()
}
pub(crate) fn notify_death() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<NotifyDeath>()
}
pub(crate) fn legal_hold() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<LegalHold>()
}
pub(crate) fn audit_retention() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<AuditRetention>()
}
pub(crate) fn approve_emergency_access() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<ApproveEmergencyAccess>()
}
pub(crate) fn wire_transfer_final() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<WireTransferFinal>()
}
pub(crate) fn freeze_note() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<FreezeNote>()
}

pub(crate) fn freeze_account_fields_read_scope(field: &str) -> WorthQueryOperationGraphReadScope {
    installed_schema()
        .installed_operation(op::<FreezeAccountFields>())
        .expect("operation installs")
        .contracts()
        .graph_reads()
        .roles()
        .iter()
        .flat_map(|role| role.read_scopes())
        .find(|scope| {
            let WorthQueryOperationGraphReadScope::NativeProjection(scope) = scope else {
                return false;
            };
            scope.entity().semantic_key() == "FixtureEntity"
                && scope.aspect().as_str() == "IdentityAspect"
                && scope.projection().mask().paths().iter().any(|path| {
                    path.fields()
                        .first()
                        .is_some_and(|candidate| candidate.as_str() == field)
                })
        })
        .cloned()
        .expect("fixture operation installs the requested field read")
}
pub(crate) fn freeze_balance() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<FreezeBalance>()
}
pub(crate) fn charge() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<Charge>()
}
