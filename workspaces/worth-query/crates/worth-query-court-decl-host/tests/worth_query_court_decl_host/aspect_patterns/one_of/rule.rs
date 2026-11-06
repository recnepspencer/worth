use super::*;
#[derive(Clone, Debug, PartialEq, Eq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Refusal {
    pub subject: String,
    pub present: u64,
}
worth_query_value_binding!(pub RefusalBinding for Refusal { identity: "native-record-patterns.refusal.v1", scalar: String, canonical: ValueDecodeBounds::new(256, 64, 8, 8), });
pub struct OneOf;
impl ApplicationInvariantMarkerIdentity<Schema> for OneOf {
    const IDENTIFIER: &'static str = "native-record-one-of";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub fn definition() -> ApplicationInvariantDefinition<Schema, OneOf> {
    ApplicationInvariantDefinition::new(
        OneOf::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [ApplicationInvariantScopeTarget::Entity("Record".into())],
            [ApplicationInvariantScopeTarget::Entity("Record".into())],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
type Field = WorthQueryApplicationInvariantFieldBinding<Schema, Record, u64>;
pub struct Rule {
    key: WorthQueryApplicationInvariantFieldBinding<Schema, Record, String>,
    first: Field,
    second: Field,
    required: Field,
    third: Field,
}
pub fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Schema>,
) -> Result<Rule, String> {
    Ok(Rule {
        key: resolver.typed_field(Key::at("Key")).ok_or("key")?,
        first: resolver
            .typed_field(FirstValue::at("FirstValue"))
            .ok_or("first")?,
        second: resolver
            .typed_field(SecondValue::at("SecondValue"))
            .ok_or("second")?,
        required: resolver
            .typed_field(SecondRequired::at("SecondRequired"))
            .ok_or("required")?,
        third: resolver
            .typed_field(ThirdValue::at("ThirdValue"))
            .ok_or("third")?,
    })
}
impl WorthQueryApplicationInvariantRule<Schema> for Rule {
    type Refusal = Refusal;
    type RefusalBinding = RefusalBinding;
    type Scope = Vec<WorthQueryApplicationInvariantEntity<Schema, Record>>;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        Some(self.prepare(planner, budget))
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<Refusal>,
            WorthQueryApplicationInvariantExecutionError,
        >,
    > {
        Some(self.check(context, scope, budget))
    }
    fn prepare_scope(
        &self,
        _: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        Err(WorthQueryApplicationInvariantPreparationError::new(
            "checked preparation required",
        ))
    }
    fn evaluate(
        &self,
        _: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        _: &Self::Scope,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Refusal>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        Err(WorthQueryApplicationInvariantExecutionError::new(
            "checked evaluation required",
        ))
    }
}
impl Rule {
    fn fields(&self) -> [&Field; 4] {
        [&self.first, &self.second, &self.required, &self.third]
    }
    fn prepare(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Result<
        Vec<WorthQueryApplicationInvariantEntity<Schema, Record>>,
        WorthQueryApplicationInvariantPreparationError,
    > {
        let stop =
            || WorthQueryApplicationInvariantPreparationError::new("original request stopped");
        if !budget.checkpoint(1) {
            return Err(stop());
        }
        let proposed = planner.proposed();
        let mut roots: Vec<WorthQueryApplicationInvariantEntity<Schema, Record>> = Vec::new();
        for field in self.fields() {
            if !budget.checkpoint(1) {
                return Err(stop());
            }
            for root in proposed
                .touched_entities(field)
                .map_err(|e| WorthQueryApplicationInvariantPreparationError::new(e.to_string()))?
            {
                if !budget.checkpoint(1) {
                    return Err(stop());
                }
                if !roots
                    .iter()
                    .any(|existing| existing.entity_id() == root.entity_id())
                {
                    if !budget.claim_result(std::mem::size_of_val(&root) as u64) {
                        return Err(stop());
                    }
                    roots.try_reserve_exact(1).map_err(|e| {
                        WorthQueryApplicationInvariantPreparationError::new(e.to_string())
                    })?;
                    roots.push(root);
                }
            }
        }
        for root in &roots {
            if !budget.checkpoint(1) {
                return Err(stop());
            }
            proposed
                .field(&self.key, root)
                .map_err(|e| WorthQueryApplicationInvariantPreparationError::new(e.to_string()))?;
            for field in self.fields() {
                if !budget.checkpoint(1) {
                    return Err(stop());
                }
                proposed.field(field, root).map_err(|e| {
                    WorthQueryApplicationInvariantPreparationError::new(e.to_string())
                })?;
            }
        }
        Ok(roots)
    }
    fn check(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &[WorthQueryApplicationInvariantEntity<Schema, Record>],
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Refusal>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        let stop = || WorthQueryApplicationInvariantExecutionError::new("original request stopped");
        if !budget.checkpoint(1) {
            return Err(stop());
        }
        for root in scope {
            if !budget.checkpoint(1) {
                return Err(stop());
            }
            let proposed = context.proposed();
            let mut values = [None; 4];
            for (value, field) in values.iter_mut().zip(self.fields()) {
                if !budget.checkpoint(1) {
                    return Err(stop());
                }
                *value = proposed.field(field, root).map_err(|e| {
                    WorthQueryApplicationInvariantExecutionError::new(e.to_string())
                })?;
            }
            let valid = matches!(
                values,
                [Some(_), None, None, None]
                    | [None, Some(_), Some(_), None]
                    | [None, None, None, Some(_)]
            );
            if !valid {
                if !budget.checkpoint(1) {
                    return Err(stop());
                }
                let subject = proposed
                    .field(&self.key, root)
                    .map_err(|e| WorthQueryApplicationInvariantExecutionError::new(e.to_string()))?
                    .ok_or_else(|| {
                        WorthQueryApplicationInvariantExecutionError::new("missing subject key")
                    })?;
                let present = [values[0], values[1], values[3]]
                    .into_iter()
                    .filter(Option::is_some)
                    .count() as u64;
                return Ok(WorthQueryApplicationInvariantVerdict::Violation(Refusal {
                    subject,
                    present,
                }));
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
