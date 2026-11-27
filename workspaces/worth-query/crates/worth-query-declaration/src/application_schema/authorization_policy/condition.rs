//! Typed field conditions on one entity of a declared ReBAC path.
//!
//! Each condition resolves its field and encodes its values through the
//! field's own record binding. Ordered comparisons exist only for bindings
//! whose stored order is their value order:
//!
//! ```compile_fail
//! use worth_query_declaration::facade::application_schema::*;
//! fn ordered<B: ApplicationOrderFaithfulBinding>() {}
//! ordered::<StringApplicationValueBinding>();
//! ```
//!
//! ```
//! use worth_query_declaration::facade::application_schema::*;
//! fn ordered<B: ApplicationOrderFaithfulBinding>() {}
//! ordered::<U64ApplicationValueBinding>();
//! ordered::<CanonicalTimestampApplicationValueBinding>();
//! ```
use std::marker::PhantomData;

use super::ApplicationAuthorizationComparison as Comparison;
use crate::application_schema::fragment::RecordSelection;
use crate::application_schema::{
    ApplicationAuthorizationRuleDenial as RuleDenial, ApplicationFieldPresence, ApplicationValue,
};

/// Marks a condition that does not read the admitted principal.
pub struct NoActor;

/// Marks a condition comparing against a field of principal entity `P`.
pub struct PrincipalActor<P>(PhantomData<fn() -> P>);

/// A condition on entity `Entity`. `Actor` names the principal it compares
/// against, so a principal-field condition only joins a path from that
/// principal.
pub struct ApplicationAuthorizationCondition<Entity, Actor = NoActor> {
    pub(super) resolved: Result<ResolvedCondition, RuleDenial>,
    marker: PhantomData<fn() -> (Entity, Actor)>,
}

#[derive(Clone, Debug)]
pub(super) struct ResolvedCondition {
    pub(super) entity: &'static str,
    pub(super) aspect: String,
    pub(super) field: String,
    /// Whether any compared field may be absent on a valid committed record.
    pub(super) optional: bool,
    /// Alternatives the authored condition lowers to; a path matches when any does.
    pub(super) alternatives: Vec<Alternative>,
    pub(super) description: String,
}

#[derive(Clone, Debug)]
pub(super) enum Alternative {
    Value(Comparison, ApplicationValue),
    Principal(Comparison, String, String),
}

mod sealed {
    pub trait Sealed {}
}

/// A binding whose encoded values order exactly as the domain values they
/// encode: the native integer bindings and `CanonicalDate`, `CanonicalTime`
/// and `CanonicalTimestamp`. Decimal, big-integer and rational text, floats,
/// strings and vocabulary codes are not order-faithful.
pub trait ApplicationOrderFaithfulBinding:
    sealed::Sealed + crate::application_schema::ApplicationScalarValueBinding
{
}

macro_rules! order_faithful {
    ($($binding:ident),+ $(,)?) => {$(
        impl sealed::Sealed for crate::application_schema::$binding {}
        impl ApplicationOrderFaithfulBinding for crate::application_schema::$binding {}
    )+};
}

order_faithful!(
    I8ApplicationValueBinding,
    I16ApplicationValueBinding,
    I32ApplicationValueBinding,
    I64ApplicationValueBinding,
    U8ApplicationValueBinding,
    U16ApplicationValueBinding,
    U32ApplicationValueBinding,
    U64ApplicationValueBinding,
    CanonicalDateApplicationValueBinding,
    CanonicalTimeApplicationValueBinding,
    CanonicalTimestampApplicationValueBinding,
);

/// Conditions authored directly on a typed record field selection.
pub trait ApplicationAuthorizationFieldConditions: RecordSelection {
    /// The field equals `value`.
    fn eq(self, value: Self::Value) -> ApplicationAuthorizationCondition<Self::Entity> {
        values(self, "=", [value], |value| {
            vec![Alternative::Value(Comparison::Equal, value)]
        })
    }

    /// The field is present and differs from `value`.
    fn ne(self, value: Self::Value) -> ApplicationAuthorizationCondition<Self::Entity> {
        values(self, "!=", [value], |value| {
            vec![
                Alternative::Value(Comparison::StrictlyLess, value.clone()),
                Alternative::Value(Comparison::StrictlyGreater, value),
            ]
        })
    }

    /// The field equals one of `values`; an empty set is refused.
    fn one_of(
        self,
        values: impl IntoIterator<Item = Self::Value>,
    ) -> ApplicationAuthorizationCondition<Self::Entity> {
        self::values(self, "one of", values, |value| {
            vec![Alternative::Value(Comparison::Equal, value)]
        })
    }

    fn lt(self, value: Self::Value) -> ApplicationAuthorizationCondition<Self::Entity>
    where
        Self::Binding: ApplicationOrderFaithfulBinding,
    {
        ordered(self, Comparison::StrictlyLess, value)
    }

    fn le(self, value: Self::Value) -> ApplicationAuthorizationCondition<Self::Entity>
    where
        Self::Binding: ApplicationOrderFaithfulBinding,
    {
        ordered(self, Comparison::AtMost, value)
    }

    fn gt(self, value: Self::Value) -> ApplicationAuthorizationCondition<Self::Entity>
    where
        Self::Binding: ApplicationOrderFaithfulBinding,
    {
        ordered(self, Comparison::StrictlyGreater, value)
    }

    fn ge(self, value: Self::Value) -> ApplicationAuthorizationCondition<Self::Entity>
    where
        Self::Binding: ApplicationOrderFaithfulBinding,
    {
        ordered(self, Comparison::AtLeast, value)
    }

    /// The field equals `principal_field` of the admitted principal.
    fn eq_actor<P>(
        self,
        principal_field: P,
    ) -> ApplicationAuthorizationCondition<Self::Entity, PrincipalActor<P::Entity>>
    where
        P: RecordSelection<Binding = Self::Binding>,
    {
        actor(self, principal_field, "=", &[Comparison::Equal])
    }

    /// The field is present and differs from `principal_field` of the
    /// admitted principal.
    fn ne_actor<P>(
        self,
        principal_field: P,
    ) -> ApplicationAuthorizationCondition<Self::Entity, PrincipalActor<P::Entity>>
    where
        P: RecordSelection<Binding = Self::Binding>,
    {
        let comparisons = [Comparison::StrictlyLess, Comparison::StrictlyGreater];
        actor(self, principal_field, "!=", &comparisons)
    }
}

impl<Field: RecordSelection> ApplicationAuthorizationFieldConditions for Field {}

fn ordered<F: RecordSelection>(
    field: F,
    comparison: Comparison,
    value: F::Value,
) -> ApplicationAuthorizationCondition<F::Entity> {
    values(field, comparison.symbol(), [value], |value| {
        vec![Alternative::Value(comparison, value)]
    })
}

fn values<F: RecordSelection, A>(
    field: F,
    operator: &str,
    values: impl IntoIterator<Item = F::Value>,
    lower: impl Fn(ApplicationValue) -> Vec<Alternative>,
) -> ApplicationAuthorizationCondition<F::Entity, A> {
    let resolved = (|| {
        let (mut condition, checked) = resolve(field, operator)?;
        let mut encoded = Vec::new();
        for value in values {
            let value = checked
                .encode(&value)
                .map_err(|refusal| RuleDenial::ValueRejected {
                    condition: condition.description.clone(),
                    cause: refusal.cause().clone(),
                })?;
            encoded.push(value);
        }
        encoded.sort();
        encoded.dedup();
        if encoded.is_empty() {
            return Err(RuleDenial::EmptyOneOf {
                condition: condition.description,
            });
        }
        condition.description = format!("{} {encoded:?}", condition.description);
        condition.alternatives = encoded.into_iter().flat_map(lower).collect();
        Ok(condition)
    })();
    ApplicationAuthorizationCondition {
        resolved,
        marker: PhantomData,
    }
}

fn actor<F: RecordSelection, P: RecordSelection, A>(
    field: F,
    principal: P,
    operator: &str,
    comparisons: &[Comparison],
) -> ApplicationAuthorizationCondition<F::Entity, A> {
    let resolved = (|| {
        let (mut condition, _) = resolve(field, operator)?;
        let (actor, _) = resolve(principal, "")?;
        condition.description = format!(
            "{} actor {}.{}",
            condition.description, actor.entity, actor.field
        );
        condition.optional |= actor.optional;
        condition.alternatives = comparisons
            .iter()
            .map(|comparison| {
                Alternative::Principal(*comparison, actor.aspect.clone(), actor.field.clone())
            })
            .collect();
        Ok(condition)
    })();
    ApplicationAuthorizationCondition {
        resolved,
        marker: PhantomData,
    }
}

type Checked<F> = crate::application_schema::fragment::RecordField<
    <F as RecordSelection>::Entity,
    <F as RecordSelection>::Aspect,
    <F as RecordSelection>::Value,
>;

fn resolve<F: RecordSelection>(
    field: F,
    operator: &str,
) -> Result<(ResolvedCondition, Checked<F>), RuleDenial> {
    let description = format!("{}.{} {operator}", field.entity(), field.field())
        .trim_end()
        .to_owned();
    let checked = field
        .checked()
        .map_err(|cause| RuleDenial::FieldUnavailable {
            condition: description.clone(),
            cause: Box::new(cause),
        })?;
    if !checked.equality_queryable() {
        return Err(RuleDenial::FieldNotQueryable {
            condition: description,
        });
    }
    let condition = ResolvedCondition {
        entity: field.entity(),
        aspect: checked.locus().aspect().to_owned(),
        field: checked.locus().field().to_owned(),
        optional: checked.presence() == ApplicationFieldPresence::Optional,
        alternatives: Vec::new(),
        description,
    };
    Ok((condition, checked))
}
