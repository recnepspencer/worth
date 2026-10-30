//! One typed field binding, completed Native contract, and installed sealing authority.
use std::any::TypeId;
use worth_query_declaration::facade::application_schema as schema;
use worth_query_installation::facade as domain;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NativeFieldBindingExpectation {
    pub(super) locus: schema::ApplicationFieldBindingLocus,
    pub(super) binding: TypeId,
    pub(super) value: TypeId,
    pub(super) identity:
        worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity,
    pub(super) scalar: domain::ScalarAspectType,
    pub(super) unit: Option<schema::ApplicationUnitIdentity>,
    pub(super) frame: Option<schema::ApplicationFrameIdentity>,
    pub(super) canonical: Option<schema::ApplicationCanonicalValueContract>,
    pub(super) presence: schema::ApplicationFieldPresence,
    pub(super) identity_name: &'static str,
}

impl NativeFieldBindingExpectation {
    #[expect(
        clippy::disallowed_methods,
        reason = "Type identity serves point lookup; canonical order uses declared identity"
    )]
    pub(super) fn of<S, E, A, F, V: 'static, W, Q, U>(
        field: schema::ApplicationFieldRef<S, E, A, F, V, W, Q, U>,
    ) -> Self
    where
        F: schema::DeclaredApplicationFieldValue<Value = V>,
        U: schema::ApplicationFieldUnit,
    {
        use schema::ApplicationScalarValueBinding;
        Self {
            locus: schema::ApplicationFieldBindingLocus::new(
                field.entity(),
                field.aspect(),
                field.field(),
            ),
            binding: TypeId::of::<F::Binding>(),
            value: TypeId::of::<V>(),
            identity: F::Binding::IDENTITY,
            scalar: F::Binding::SCALAR_FAMILY,
            unit: F::Binding::UNIT,
            frame: F::Binding::FRAME,
            canonical: F::Binding::canonical_contract(),
            presence: F::PRESENCE,
            identity_name: F::Binding::IDENTITY_NAME,
        }
    }
}

#[derive(Debug)]
pub(super) struct NativeFieldContractDenial {
    pub(super) locus: schema::ApplicationFieldBindingLocus,
    pub(super) detail: String,
}
impl NativeFieldContractDenial {
    #[cfg(test)]
    pub(super) fn detail(&self) -> &str {
        &self.detail
    }
    #[cfg(test)]
    pub(super) fn field_locus(&self) -> Option<&schema::ApplicationFieldBindingLocus> {
        Some(&self.locus)
    }
}
fn field_denial(
    locus: schema::ApplicationFieldBindingLocus,
    detail: &str,
) -> NativeFieldContractDenial {
    NativeFieldContractDenial {
        locus,
        detail: detail.into(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FieldExpectation {
    pub(super) binding: NativeFieldBindingExpectation,
    pub(super) aspect: domain::AspectContract,
    pub(super) native_binding: domain::AspectBinding,
}

impl FieldExpectation {
    pub(super) fn declared<S, E, A, F, V: 'static, W, Q, U>(
        declaration: &schema::ErasedApplicationSchemaDeclaration,
        field: schema::ApplicationFieldRef<S, E, A, F, V, W, Q, U>,
    ) -> Result<Self, NativeFieldContractDenial>
    where
        F: schema::DeclaredApplicationFieldValue<Value = V>,
        U: schema::ApplicationFieldUnit,
    {
        NativeFieldBindingExpectation::of(field).declare(declaration)
    }
}

impl NativeFieldBindingExpectation {
    pub(super) fn declare(
        &self,
        declaration: &schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<FieldExpectation, NativeFieldContractDenial> {
        let locus = &self.locus;
        let fail = |detail: &str| field_denial(locus.clone(), detail);
        let canonical = self.canonical.clone();
        let member = declaration
            .members()
            .iter()
            .find(|member| {
                matches!(member,
            schema::ApplicationSchemaMember::Field { entity,aspect,field: name,.. }
                if entity == locus.entity() && aspect == locus.aspect() && name == locus.field())
            })
            .ok_or_else(|| fail("native field is absent from the completed declaration"))?;
        let schema::ApplicationSchemaMember::Field {
            presence,
            scalar_family,
            value_type,
            canonical: actual_canonical,
            unit,
            frame,
            ..
        } = member
        else {
            unreachable!()
        };
        if *presence != self.presence
            || *scalar_family != self.scalar
            || value_type != self.identity_name
            || *actual_canonical != canonical
            || unit.as_deref() != self.unit.map(|unit| unit.as_str())
            || frame.as_deref() != self.frame.map(|frame| frame.as_str())
        {
            return Err(fail(
                "native field binding differs from its actual declaration",
            ));
        }
        let aspect =
            domain::declared_application_aspect(declaration, locus.entity(), locus.aspect())
                .map_err(|_| fail("native aspect cannot form its complete native contract"))?;
        Ok(FieldExpectation {
            binding: self.clone(),
            native_binding: aspect.binding().clone(),
            aspect: aspect.contract().clone(),
        })
    }
}

impl FieldExpectation {
    pub(super) fn seal<S: schema::ApplicationSchema>(
        &self,
        installed: &domain::WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), NativeFieldContractDenial> {
        self.seal_components(
            installed.value_bindings().field(
                self.binding.locus.entity(),
                self.binding.locus.aspect(),
                self.binding.locus.field(),
            ),
            installed
                .native_contracts()
                .aspect(self.binding.locus.entity(), self.binding.locus.aspect()),
        )
    }

    pub(super) fn seal_components(
        &self,
        binding: Option<&domain::WorthQueryInstalledApplicationValueBinding>,
        native: Option<&domain::WorthQueryInstalledApplicationAspectContract>,
    ) -> Result<(), NativeFieldContractDenial> {
        let fail = |detail| field_denial(self.binding.locus.clone(), detail);
        let binding = binding.ok_or_else(|| fail("native field has no installed binding"))?;
        if binding.binding_type() != self.binding.binding
            || binding.value_type() != self.binding.value
            || binding.identity() != &self.binding.identity
            || binding.scalar_family() != self.binding.scalar
            || binding.unit() != self.binding.unit
            || binding.frame() != self.binding.frame
            || binding.canonical_contract() != self.binding.canonical.as_ref()
        {
            return Err(fail(
                "native field differs from its installed value binding",
            ));
        }
        if native.is_none_or(|aspect| {
            aspect.contract() != &self.aspect || aspect.binding() != &self.native_binding
        }) {
            return Err(fail(
                "native field differs from its complete installed aspect",
            ));
        }
        Ok(())
    }
}
