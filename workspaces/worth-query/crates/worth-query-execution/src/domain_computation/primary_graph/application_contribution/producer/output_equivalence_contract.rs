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
/// Its entity, value and scalar codec are checked by the selector type.
/// Column presence is checked against the owning record before construction.
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
    pub fn exact_u64<F>(field: F) -> Result<Self, schema::ApplicationSchemaDeclarationDenial>
    where
        F: schema::fragment::RecordSelection<
            Entity = Role::Entity,
            Value = u64,
            Binding = schema::U64ApplicationValueBinding,
        >,
    {
        let metadata = field.checked()?;
        if metadata.presence() != schema::ApplicationFieldPresence::Required {
            return Err(
                schema::ApplicationSchemaDeclarationDenial::InvalidRecordField {
                    field: field.expected_recipe().locus().clone(),
                },
            );
        }
        Ok(Self {
            field: NativeFieldBindingExpectation::of(field)?,
            marker: PhantomData,
        })
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
    use crate::equality_type_id::EqualityTypeId as TypeId;
    (field.binding == TypeId::of::<schema::U64ApplicationValueBinding>().into_external()
        && field.value == TypeId::of::<u64>().into_external()
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
