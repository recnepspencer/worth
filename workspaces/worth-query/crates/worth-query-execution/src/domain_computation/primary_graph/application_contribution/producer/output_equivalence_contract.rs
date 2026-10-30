//! Opt-in producer field scope; comparison and disclosure require their own evidence.
use std::{marker::PhantomData, sync::Arc};
use worth_query_declaration::facade::application_operation::{
    application_value_identity, ApplicationValueIdentityDomain, WorthQueryApplicationOutputRole,
    WorthQueryExactlyOneOutput, WorthQueryPreserveOutput,
};
use worth_query_declaration::facade::application_schema as schema;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::super::native_field_contract::{FieldExpectation, NativeFieldBindingExpectation};
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

/// Exact comparison of one required standard u64 field on this exactly-one Preserve role.
/// This declares scope only. Missing pre-evaluation field presence, reuse and
/// restoration cannot establish an equivalent-output evaluation.
///
/// The field's entity is the role's entity (positive construction):
/// ```
/// use worth_query_execution::facade::application_contribution::WorthQueryProducerOutputEquivalence as Contract;
/// use worth_query_declaration::facade::{application_operation::*, application_schema::*};
/// fn declared<S, R, A, F, W, Q, U>(
///     field: ApplicationFieldRef<S, R::Entity, A, F, u64, W, Q, U>,
/// ) -> Contract<S, R>
/// where R: WorthQueryApplicationOutputRole<Schema=S, Cardinality=WorthQueryExactlyOneOutput, Action=WorthQueryPreserveOutput>,
/// F: RequiredApplicationFieldValue<Value=u64, Binding=U64ApplicationValueBinding>,
/// U: ApplicationFieldUnit,
/// { Contract::exact_u64(field) }
/// ```
/// A sibling entity cannot be substituted:
/// ```compile_fail,E0308
/// use worth_query_execution::facade::application_contribution::WorthQueryProducerOutputEquivalence as Contract;
/// use worth_query_declaration::facade::{application_operation::*, application_schema::*};
/// fn wrong_entity<S, R, E, A, F, W, Q, U>(
///     field: ApplicationFieldRef<S, E, A, F, u64, W, Q, U>,
/// ) -> Contract<S, R>
/// where R: WorthQueryApplicationOutputRole<Schema=S, Cardinality=WorthQueryExactlyOneOutput, Action=WorthQueryPreserveOutput>,
/// F: RequiredApplicationFieldValue<Value=u64, Binding=U64ApplicationValueBinding>,
/// U: ApplicationFieldUnit,
/// { Contract::exact_u64(field) }
/// ```
/// An optional field cannot acquire a present-field contract:
/// ```compile_fail,E0277
/// use worth_query_execution::facade::application_contribution::WorthQueryProducerOutputEquivalence as Contract;
/// use worth_query_declaration::facade::{application_operation::*, application_schema::*};
/// fn optional<S, R, A, F, W, Q, U>(
///     field: ApplicationFieldRef<S, R::Entity, A, F, u64, W, Q, U>,
/// ) -> Contract<S, R>
/// where R: WorthQueryApplicationOutputRole<Schema=S, Cardinality=WorthQueryExactlyOneOutput, Action=WorthQueryPreserveOutput>,
/// F: OptionalApplicationFieldValue<Value=u64, Binding=U64ApplicationValueBinding>,
/// U: ApplicationFieldUnit,
/// { Contract::exact_u64(field) }
/// ```
pub struct WorthQueryProducerOutputEquivalence<Schema, Role> {
    field: NativeFieldBindingExpectation,
    marker: PhantomData<fn() -> (Schema, Role)>,
}

impl<Schema, Role> WorthQueryProducerOutputEquivalence<Schema, Role>
where
    Role: WorthQueryApplicationOutputRole<
        Schema = Schema,
        Cardinality = WorthQueryExactlyOneOutput,
        Action = WorthQueryPreserveOutput,
    >,
{
    pub fn exact_u64<A, F, W, Q, U>(
        field: schema::ApplicationFieldRef<Schema, Role::Entity, A, F, u64, W, Q, U>,
    ) -> Self
    where
        F: schema::RequiredApplicationFieldValue<
            Value = u64,
            Binding = schema::U64ApplicationValueBinding,
        >,
        U: schema::ApplicationFieldUnit,
    {
        Self {
            field: NativeFieldBindingExpectation::of(field),
            marker: PhantomData,
        }
    }
}

impl<Schema, Role> WorthQueryProducerOutputEquivalence<Schema, Role> {
    pub(super) fn into_field(self) -> NativeFieldBindingExpectation {
        self.field
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in super::super) struct InstalledProducerOutputEquivalence {
    pub(in super::super) field: FieldExpectation,
    pub(in super::super) meaning: [u8; 32],
}

pub(in super::super) fn seal<Schema: schema::ApplicationSchema>(
    identity: &str,
    role: &str,
    field: &NativeFieldBindingExpectation,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<Arc<InstalledProducerOutputEquivalence>, WorthQueryPrimaryGraphInstallationDenial> {
    seal_contract(
        identity,
        role,
        field,
        installed.installed_declaration(),
        installed.value_bindings(),
        installed.native_contracts(),
    )
}

fn seal_contract(
    identity: &str,
    role: &str,
    field: &NativeFieldBindingExpectation,
    declaration: &schema::ErasedApplicationSchemaDeclaration,
    values: &worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    contracts: &worth_query_installation::facade::WorthQueryInstalledApplicationSchemaContractCatalog,
) -> Result<Arc<InstalledProducerOutputEquivalence>, WorthQueryPrimaryGraphInstallationDenial> {
    let fail = |detail: &str| {
        WorthQueryPrimaryGraphInstallationDenial::new(
            WorthQueryPrimaryGraphInstallationDenialKind::ProducerBindingMeaningMismatch,
            format!("{identity}: {detail}"),
        )
    };
    let portable = standard_u64_meaning(field)
        .ok_or_else(|| fail("output equivalence requires a required standard u64 field"))?;
    let declared = field
        .declare(declaration)
        .map_err(|denial| fail(&denial.detail))?;
    let native = contracts.aspect(field.locus.entity(), field.locus.aspect());
    declared
        .seal_components(
            values.field(
                field.locus.entity(),
                field.locus.aspect(),
                field.locus.field(),
            ),
            native,
        )
        .map_err(|denial| fail(&denial.detail))?;
    let native = native.expect("field sealing validated the actual installed aspect");
    let meaning = application_value_identity(
        ApplicationValueIdentityDomain::ProducerImplementationEdition,
        identity,
        &(
            "producer-output-equivalence.present-standard-u64-exact.v1",
            role,
            portable,
            native.canonical_contract_material(),
            native.binding(),
        ),
    )
    .map_err(|_| fail("output equivalence meaning cannot be encoded"))?
    .identity();
    Ok(Arc::new(InstalledProducerOutputEquivalence {
        field: declared,
        meaning,
    }))
}

fn standard_u64_meaning(
    field: &NativeFieldBindingExpectation,
) -> Option<impl serde::Serialize + '_> {
    use std::any::TypeId;
    (field.binding == TypeId::of::<schema::U64ApplicationValueBinding>()
        && field.value == TypeId::of::<u64>()
        && field.scalar == worth_foundational::facade::ScalarAspectType::UInt64
        && field.presence == schema::ApplicationFieldPresence::Required
        && field.canonical.is_none())
    .then_some((
        &field.locus,
        field.identity.as_str(),
        field.scalar,
        field.unit.map(|unit| unit.as_str()),
        field.frame.map(|frame| frame.as_str()),
    ))
}
