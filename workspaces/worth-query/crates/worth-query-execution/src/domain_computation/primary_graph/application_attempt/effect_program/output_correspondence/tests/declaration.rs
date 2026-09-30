//! The test schema, the output contracts under test, and their role markers.
//! A test that needs a use its contract does not declare cannot write one:
//! it builds the erased use directly with [`stray`], the form a readmitted or
//! portable role takes.

use std::any::TypeId;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationOutputContract, ApplicationMutationOutputPosture,
    ApplicationMutationOutputPostureSet, ApplicationMutationOutputRoleCardinality,
    ApplicationMutationOutputRoleDescriptor, ApplicationMutationOutputRoleFamilyDescriptor,
    WorthQueryApplicationDeclaredOutputRole as Declared,
    WorthQueryApplicationDeclaredOutputRoleFamily as DeclaredFamily,
    WorthQueryApplicationOutputRole, WorthQueryApplicationOutputRoleFamily,
    WorthQueryAtMostOneOutput, WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
    WorthQueryPreserveOutput, WorthQueryRetireOutput,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationSchema, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationDenial,
};

use super::super::OutputRoleUse;

pub(super) struct Schema;

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.tests";
    const NAME: &'static str = "output-correspondence";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        unreachable!("the output-correspondence proof needs no schema installation")
    }
}

pub(super) struct Entity;
pub(super) struct WrongEntity;

impl ApplicationEntityMarkerIdentity<Schema> for Entity {
    const IDENTIFIER: &'static str = "entity";
}

impl ApplicationEntityMarkerIdentity<Schema> for WrongEntity {
    const IDENTIFIER: &'static str = "wrong-entity";
}

macro_rules! role {
    ($marker:ident: $contract:ty, $action:ty, $cardinality:ty, $name:literal) => {
        pub(super) struct $marker;
        impl WorthQueryApplicationOutputRole for $marker {
            type Schema = Schema;
            type Contract = $contract;
            type Entity = Entity;
            type Action = $action;
            type Cardinality = $cardinality;
            const NAME: &'static str = $name;
        }
    };
}

macro_rules! family {
    ($marker:ident: $contract:ty, $prefix:literal, $postures:expr, $minimum:literal) => {
        pub(super) struct $marker;
        impl WorthQueryApplicationOutputRoleFamily for $marker {
            type Schema = Schema;
            type Contract = $contract;
            type Entity = Entity;
            const PREFIX: &'static str = $prefix;
            const POSTURES: ApplicationMutationOutputPostureSet = $postures;
            const MINIMUM: usize = $minimum;
        }
    };
}

macro_rules! contract {
    ($contract:ident: [$($role:ty),*], [$($family:ty),*]) => {
        pub(super) struct $contract;
        impl ApplicationMutationOutputContract<Schema> for $contract {
            const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
                &[$(<$role as Declared>::DESCRIPTOR),*];
            const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
                &[$(<$family as DeclaredFamily>::DESCRIPTOR),*];
        }
    };
}

contract!(Outputs: [Preserved, Created, Retired], []);
role!(Preserved: Outputs, WorthQueryPreserveOutput, WorthQueryExactlyOneOutput, "preserved");
role!(Created: Outputs, WorthQueryCreateOutput, WorthQueryExactlyOneOutput, "created");
role!(Retired: Outputs, WorthQueryRetireOutput, WorthQueryExactlyOneOutput, "retired");

contract!(PreserveOutputs: [OnlyPreserved], []);
role!(OnlyPreserved: PreserveOutputs, WorthQueryPreserveOutput, WorthQueryExactlyOneOutput, "preserved");

contract!(CreateOutputs: [OnlyCreated], []);
role!(OnlyCreated: CreateOutputs, WorthQueryCreateOutput, WorthQueryExactlyOneOutput, "created");

contract!(RetireOutputs: [OnlyRetired], []);
role!(OnlyRetired: RetireOutputs, WorthQueryRetireOutput, WorthQueryExactlyOneOutput, "retired");

contract!(OptionalOutputs: [Subject, Companion], [Member]);
role!(Subject: OptionalOutputs, WorthQueryPreserveOutput, WorthQueryExactlyOneOutput, "preserved");
role!(Companion: OptionalOutputs, WorthQueryPreserveOutput, WorthQueryAtMostOneOutput, "companion");
family!(Member: OptionalOutputs, "member.", ApplicationMutationOutputPostureSet::PRESERVE, 0);

contract!(FaceOutputs: [], [Face]);
family!(Face: FaceOutputs, "face.", ApplicationMutationOutputPostureSet::CREATE, 2);

contract!(MixedOutputs: [Unrelated], [Mixed]);
role!(Unrelated: MixedOutputs, WorthQueryPreserveOutput, WorthQueryExactlyOneOutput, "unrelated");
family!(Mixed: MixedOutputs, "family.", ApplicationMutationOutputPostureSet::ALL, 0);

contract!(ForeignOutputs: [Foreign], []);
role!(Foreign: ForeignOutputs, WorthQueryPreserveOutput, WorthQueryExactlyOneOutput, "foreign");

const _: () = {
    let () = <Preserved as Declared>::DECLARED;
    let () = <Created as Declared>::DECLARED;
    let () = <Retired as Declared>::DECLARED;
    let () = <Companion as Declared>::DECLARED;
    let () = <Face as DeclaredFamily>::DECLARED;
    let () = <Mixed as DeclaredFamily>::DECLARED;
};

/// An erased use of `name` on `Contract` that no marker declares: the only
/// way such a use reaches Query is a readmitted or portable role.
pub(super) fn stray<Contract: 'static, Target>(
    name: &str,
    posture: ApplicationMutationOutputPosture,
    cardinality: ApplicationMutationOutputRoleCardinality,
) -> OutputRoleUse
where
    Target: ApplicationEntityMarkerIdentity<Schema> + 'static,
{
    OutputRoleUse {
        name: name.to_owned(),
        posture,
        cardinality,
        entity_name: Target::IDENTIFIER,
        entity_type: TypeId::of::<Target>(),
        contract_type: TypeId::of::<Contract>(),
    }
}
