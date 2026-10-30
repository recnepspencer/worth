//! Typed field meaning flows from the completed declaration to installed sealing.
use super::super::super::native_field_contract::FieldExpectation;
use super::super::WorthQueryApplicationConditionalPackageContract;
use super::*;
use worth_query_declaration::facade::application_schema as schema;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryReadinessFieldUse {
    Delivery,
    Eligibility,
    DeliveryAndEligibility,
}

impl FieldExpectation {
    fn projection(
        &self,
    ) -> Result<
        domain::WorthQueryOperationNativeProjectionContract,
        WorthQueryOutputReadinessContractDenial,
    > {
        domain::WorthQueryOperationNativeProjectionContract::new(self.aspect.clone(), self.mask())
            .map_err(|_| {
                field_denial(
                    self.binding.locus.clone(),
                    "readiness field projection is invalid",
                )
            })
    }
    fn mask(&self) -> domain::AspectMask<domain::ProjectionMask> {
        if matches!(
            self.aspect.shape(),
            worth_foundational::facade::AspectShape::Scalar(_)
        ) {
            return domain::AspectMask::whole_aspect();
        }
        domain::AspectMask::new([domain::CanonicalFieldPath::single(
            domain::FieldKey::new(self.binding.locus.field()).expect("declared field key"),
        )])
    }
    fn dependency(
        &self,
    ) -> Result<domain::WorthQuerySemanticTruthDependency, WorthQueryOutputReadinessContractDenial>
    {
        let change = if matches!(
            self.aspect.shape(),
            worth_foundational::facade::AspectShape::Scalar(_)
        ) {
            domain::AuthoritativeAspectChangeKind::WholeAspectSet
        } else {
            domain::AuthoritativeAspectChangeKind::FieldSet
        };
        domain::WorthQuerySemanticTruthDependency::new(
            domain::WorthQueryConditionalGraphReadRole::new("primary").expect("static role"),
            self.aspect.clone(),
            self.mask(),
            self.native_binding.clone(),
            domain::WorthQuerySemanticLocality::SourceRecord,
            [change],
        )
        .map_err(|_| {
            field_denial(
                self.binding.locus.clone(),
                "readiness field dependency is invalid",
            )
        })
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
            self.delivery_fields.insert(expected.binding.locus.clone());
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
            .sort_by(|left, right| left.binding.locus.cmp(&right.binding.locus));
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
        if !self.delivery_fields.contains(&expected.binding.locus) {
            return Err(field_denial(
                expected.binding.locus,
                "readiness field is not a declared delivery input",
            ));
        }
        if self.definition.semantics().conditional_nodes.as_slice() != std::slice::from_ref(node) {
            return Err(field_denial(
                expected.binding.locus,
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
                expected.binding.locus.clone(),
                "readiness delivery dependency is missing",
            )
        })?;
        if matches.next().is_some() {
            return Err(field_denial(
                expected.binding.locus,
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
