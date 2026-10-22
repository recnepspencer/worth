//! Typed field meaning flows from the completed declaration to installed sealing.
use super::super::WorthQueryApplicationConditionalPackageContract;
use super::*;
use std::any::TypeId;
use worth_query_declaration::facade::application_schema as schema;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryReadinessFieldUse {
    Delivery,
    Eligibility,
    DeliveryAndEligibility,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in super::super) struct FieldExpectation {
    locus: schema::ApplicationFieldBindingLocus,
    binding: TypeId,
    value: TypeId,
    identity: worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity,
    scalar: domain::ScalarAspectType,
    unit: Option<schema::ApplicationUnitIdentity>,
    frame: Option<schema::ApplicationFrameIdentity>,
    canonical: Option<schema::ApplicationCanonicalValueContract>,
    aspect: domain::AspectContract,
}

impl FieldExpectation {
    fn declared<S, E, A, F, V: 'static, W, Q, U>(
        declaration: &schema::ErasedApplicationSchemaDeclaration,
        field: schema::ApplicationFieldRef<S, E, A, F, V, W, Q, U>,
    ) -> Result<Self, WorthQueryOutputReadinessContractDenial>
    where
        F: schema::DeclaredApplicationFieldValue<Value = V>,
        U: schema::ApplicationFieldUnit,
    {
        use schema::ApplicationScalarValueBinding;
        let locus = schema::ApplicationFieldBindingLocus::new(
            field.entity(),
            field.aspect(),
            field.field(),
        );
        let fail = |detail: &str| field_denial(locus.clone(), detail);
        let canonical = F::Binding::canonical_contract();
        let member = declaration
            .members()
            .iter()
            .find(|member| {
                matches!(member,
            schema::ApplicationSchemaMember::Field { entity,aspect,field: name,.. }
                if entity == locus.entity() && aspect == locus.aspect() && name == locus.field())
            })
            .ok_or_else(|| fail("readiness field is absent from the completed declaration"))?;
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
        if *presence != F::PRESENCE
            || *scalar_family != F::Binding::SCALAR_FAMILY
            || value_type != F::Binding::IDENTITY_NAME
            || *actual_canonical != canonical
            || unit.as_deref() != F::Binding::UNIT.map(|unit| unit.as_str())
            || frame.as_deref() != F::Binding::FRAME.map(|frame| frame.as_str())
        {
            return Err(fail(
                "readiness field binding differs from its actual declaration",
            ));
        }
        let aspect = domain::declared_application_aspect_contract(
            declaration,
            locus.entity(),
            locus.aspect(),
        )
        .map_err(|_| fail("readiness aspect cannot form its complete native contract"))?;
        Ok(Self {
            locus,
            binding: TypeId::of::<F::Binding>(),
            value: TypeId::of::<V>(),
            identity: F::Binding::IDENTITY,
            scalar: F::Binding::SCALAR_FAMILY,
            unit: F::Binding::UNIT,
            frame: F::Binding::FRAME,
            canonical,
            aspect,
        })
    }

    pub(in super::super) fn seal<S: schema::ApplicationSchema>(
        &self,
        installed: &domain::WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), WorthQueryOutputReadinessContractDenial> {
        let fail = |detail| field_denial(self.locus.clone(), detail);
        let binding = installed
            .value_bindings()
            .field(self.locus.entity(), self.locus.aspect(), self.locus.field())
            .ok_or_else(|| fail("readiness field has no installed binding"))?;
        if binding.binding_type() != self.binding
            || binding.value_type() != self.value
            || binding.identity() != &self.identity
            || binding.scalar_family() != self.scalar
            || binding.unit() != self.unit
            || binding.frame() != self.frame
            || binding.canonical_contract() != self.canonical.as_ref()
        {
            return Err(fail(
                "readiness field differs from its installed value binding",
            ));
        }
        if installed
            .native_contracts()
            .aspect(self.locus.entity(), self.locus.aspect())
            .is_none_or(|aspect| aspect.contract() != &self.aspect)
        {
            return Err(fail(
                "readiness field differs from its complete installed aspect",
            ));
        }
        Ok(())
    }

    fn projection(
        &self,
    ) -> Result<
        domain::WorthQueryOperationNativeProjectionContract,
        WorthQueryOutputReadinessContractDenial,
    > {
        domain::WorthQueryOperationNativeProjectionContract::new(self.aspect.clone(), self.mask())
            .map_err(|_| field_denial(self.locus.clone(), "readiness field projection is invalid"))
    }
    fn mask(&self) -> domain::AspectMask<domain::ProjectionMask> {
        domain::AspectMask::new([domain::CanonicalFieldPath::single(
            domain::FieldKey::new(self.locus.field()).expect("declared field key"),
        )])
    }
    fn dependency(
        &self,
    ) -> Result<domain::WorthQuerySemanticTruthDependency, WorthQueryOutputReadinessContractDenial>
    {
        domain::WorthQuerySemanticTruthDependency::new(
            domain::WorthQueryConditionalGraphReadRole::new("primary").expect("static role"),
            self.aspect.clone(),
            self.mask(),
            domain::AspectBinding::EntityField {
                field: domain::FieldKey::new(self.locus.aspect()).expect("declared aspect key"),
            },
            domain::WorthQuerySemanticLocality::SourceRecord,
            [domain::AuthoritativeAspectChangeKind::FieldSet],
        )
        .map_err(|_| field_denial(self.locus.clone(), "readiness field dependency is invalid"))
    }
}

impl WorthQueryOutputReadinessContractBuilder {
    #[allow(clippy::too_many_arguments)]
    pub fn for_field<S, E, A, F, V: 'static, W, Q, U>(
        declaration: &schema::ErasedApplicationSchemaDeclaration,
        primary: schema::ApplicationFieldRef<S, E, A, F, V, W, Q, U>,
        identity: domain::WorthQueryDomainOperationIdentity,
        node: impl Into<String>,
        query: CanonicalQueryBundle,
        role: domain::WorthQueryOperationProjectionRole,
        strategy: domain::WorthQueryExecutionStrategyName,
        family: impl Into<String>,
    ) -> Result<Self, WorthQueryOutputReadinessContractDenial>
    where
        F: schema::DeclaredApplicationFieldValue<Value = V>,
        U: schema::ApplicationFieldUnit,
    {
        let expected = FieldExpectation::declared(declaration, primary)?;
        let mut builder = Self::new(
            identity,
            node,
            expected.projection()?,
            query,
            role,
            strategy,
            family,
        );
        builder.fields.push(expected);
        Ok(builder)
    }

    pub fn field_dependency<S, E, A, F, V: 'static, W, Q, U>(
        mut self,
        declaration: &schema::ErasedApplicationSchemaDeclaration,
        field: schema::ApplicationFieldRef<S, E, A, F, V, W, Q, U>,
        usage: WorthQueryReadinessFieldUse,
    ) -> Result<Self, WorthQueryOutputReadinessContractDenial>
    where
        F: schema::DeclaredApplicationFieldValue<Value = V>,
        U: schema::ApplicationFieldUnit,
    {
        let expected = FieldExpectation::declared(declaration, field)?;
        let dependency = expected.dependency()?;
        self.semantic_reads.push(expected.projection()?);
        self.dependencies.push(dependency.clone());
        if usage != WorthQueryReadinessFieldUse::Eligibility {
            self.delivery_fields.insert(expected.locus.clone());
        }
        if usage != WorthQueryReadinessFieldUse::Delivery {
            self.readiness_dependencies.push(dependency);
        }
        self.fields.push(expected);
        Ok(self)
    }

    pub fn build_typed<D, O, F>(
        mut self,
    ) -> Result<WorthQueryOutputReadinessContract<D, O, F>, WorthQueryOutputReadinessContractDenial>
    {
        if self.fields.is_empty() || self.delivery_fields.is_empty() {
            return Err(denial(
                "typed readiness requires actual declared delivery fields",
            ));
        }
        self.fields
            .sort_by(|left, right| left.locus.cmp(&right.locus));
        self.fields.dedup();
        let fields = std::mem::take(&mut self.fields);
        let delivery_fields = std::mem::take(&mut self.delivery_fields);
        let node = self.node_identity.clone();
        Ok(WorthQueryOutputReadinessContract {
            definition: self.build()?,
            fields,
            delivery_fields,
            node,
        })
    }
}

pub struct WorthQueryOutputReadinessContract<D, O, F> {
    definition: domain::WorthQueryDomainOperationDefinition<D, O, F>,
    fields: Vec<FieldExpectation>,
    delivery_fields: std::collections::BTreeSet<schema::ApplicationFieldBindingLocus>,
    node: String,
}
impl<D, O, F> WorthQueryOutputReadinessContract<D, O, F> {
    pub fn definition(&self) -> &domain::WorthQueryDomainOperationDefinition<D, O, F> {
        &self.definition
    }
    pub fn into_package_contract(
        self,
        binding: domain::WorthQueryPortableApplicationConditionalOperationBinding,
    ) -> WorthQueryApplicationConditionalPackageContract {
        let mut contract = WorthQueryApplicationConditionalPackageContract::new(
            self.definition.into_portable(),
            binding,
            self.node,
        );
        contract.fields = self.fields;
        contract.delivery_fields = self.delivery_fields;
        contract
    }
    /// Resolve a declared delivery field only after canonical ordering.
    pub fn delivery_ordinal<S, E, A, Field, V: 'static, W, Q, U>(
        &self,
        installed: &domain::WorthQueryInstalledApplicationSchema<S>,
        node: &domain::WorthQueryPortableConditionalNodeDeclaration,
        field: schema::ApplicationFieldRef<S, E, A, Field, V, W, Q, U>,
    ) -> Result<usize, WorthQueryOutputReadinessContractDenial>
    where
        S: schema::ApplicationSchema,
        Field: schema::DeclaredApplicationFieldValue<Value = V>,
        U: schema::ApplicationFieldUnit,
    {
        let expected = FieldExpectation::declared(installed.installed_declaration(), field)?;
        if !self.delivery_fields.contains(&expected.locus) {
            return Err(field_denial(
                expected.locus,
                "readiness field is not a declared delivery input",
            ));
        }
        if self.definition.semantics().conditional_nodes.as_slice() != std::slice::from_ref(node) {
            return Err(field_denial(
                expected.locus,
                "readiness node differs from its generated contract",
            ));
        }
        expected.seal(installed)?;
        let dependency = expected.dependency()?;
        let mut matches = node
            .dependencies()
            .iter()
            .enumerate()
            .filter_map(|(ordinal, value)| (value == &dependency).then_some(ordinal));
        let ordinal = matches.next().ok_or_else(|| {
            field_denial(
                expected.locus.clone(),
                "readiness delivery dependency is missing",
            )
        })?;
        if matches.next().is_some() {
            return Err(field_denial(
                expected.locus,
                "readiness delivery dependency is ambiguous",
            ));
        }
        Ok(ordinal)
    }
}

fn field_denial(
    locus: schema::ApplicationFieldBindingLocus,
    detail: &str,
) -> WorthQueryOutputReadinessContractDenial {
    WorthQueryOutputReadinessContractDenial {
        detail: detail.into(),
        locus: Some(locus),
    }
}
