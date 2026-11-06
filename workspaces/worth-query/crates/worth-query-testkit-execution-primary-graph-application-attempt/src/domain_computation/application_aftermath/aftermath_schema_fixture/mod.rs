//! Minimal application schema that installs aftermath through the production door.

use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_declaration::facade::application_schema::ApplicationOperationRef;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::WorthQueryInstallationAdmissionProfile;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
use worth_query_installation::facade::WorthQueryInstallationRuntimeIdentity;
use worth_query_installation::facade::WorthQueryInstalledAftermathContract;
use worth_query_installation::facade::WorthQueryInstalledPackageIndex;
use worth_query_installation::facade::WorthQueryOperationGraphReadScope;
use worth_query_installation::facade::WorthQueryPortableDomainIdentity;
use worth_query_installation::facade::WorthQueryPortableDomainPackage;

use crate::domain_computation::application_aftermath::aftermath_schema_fixture::declaration::AftermathFixtureSchema;
use crate::domain_computation::application_aftermath::aftermath_schema_fixture::declaration::Charge;
use crate::domain_computation::application_aftermath::aftermath_schema_fixture::declaration::FixtureInput;
use crate::domain_computation::application_aftermath::aftermath_schema_fixture::declaration::FreezeAccount;
use crate::domain_computation::application_aftermath::aftermath_schema_fixture::declaration::FreezeAccountFields;
use crate::domain_computation::application_aftermath::aftermath_schema_fixture::declaration::FreezeAccountFieldsReversed;
use crate::domain_computation::application_aftermath::aftermath_schema_fixture::declaration::FreezeBalance;
use crate::domain_computation::application_aftermath::aftermath_schema_fixture::declaration::FreezeNote;
use crate::domain_computation::application_aftermath::aftermath_schema_fixture::declaration::NotifyDeath;
use crate::domain_computation::application_aftermath::aftermath_schema_fixture::declaration::Transfer;

pub fn op<Operation>() -> ApplicationOperationRef<AftermathFixtureSchema, Operation, FixtureInput>
where
    Operation: ApplicationOperationMarkerIdentity<AftermathFixtureSchema> + 'static,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = FixtureInput>,
{
    ApplicationOperationRef::from_declaration()
}

pub fn installed_schema(
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

pub fn aftermath_of<Operation>() -> WorthQueryInstalledAftermathContract
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

pub fn transfer() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<Transfer>()
}

pub fn freeze_account() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<FreezeAccount>()
}
pub fn freeze_account_fields() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<FreezeAccountFields>()
}
pub fn freeze_account_fields_reversed() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<FreezeAccountFieldsReversed>()
}
pub fn notify_death() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<NotifyDeath>()
}

pub fn freeze_note() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<FreezeNote>()
}

pub fn aspect_name() -> &'static str {
    declaration::FrozenField::at("frozen").aspect()
}

pub fn freeze_account_fields_read_scope(field: &str) -> WorthQueryOperationGraphReadScope {
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
                && scope.aspect().as_str() == aspect_name()
                && scope.projection().mask().paths().iter().any(|path| {
                    path.fields()
                        .first()
                        .is_some_and(|candidate| candidate.as_str() == field)
                })
        })
        .cloned()
        .expect("fixture operation installs the requested field read")
}
pub fn freeze_balance() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<FreezeBalance>()
}
pub fn charge() -> WorthQueryInstalledAftermathContract {
    aftermath_of::<Charge>()
}

#[path = "declaration/mod.rs"]
pub mod declaration;
#[path = "operations/mod.rs"]
pub mod operations;
