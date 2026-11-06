//! A typed path selects a field from the owning record at schema build.

use super::super::{
    ApplicationFieldBindingLocus, ApplicationOperationDecisionReadTarget,
    ApplicationOperationProgramTarget, ApplicationOperationRef,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember,
};
use super::super::{
    ApplicationFieldBindingRecipe, ApplicationScalarValueBinding, ApplicationValue,
    ApplicationValueEncodeDenial,
};
use super::{Aspect, Entity, Fragment, In, Record, Scalar, Shape};
use std::any::TypeId;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct RecordField<Entity, Aspect, Value> {
    locus: ApplicationFieldBindingLocus,
    path: String,
    recipe: ApplicationFieldBindingRecipe,
    content: Option<super::super::ApplicationContentBindingRecipe>,
    equality: bool,
    writable: bool,
    presence: super::super::ApplicationFieldPresence,
    encode: fn(&Value) -> Result<ApplicationValue, ApplicationValueEncodeDenial>,
    decode: fn(
        &ApplicationValue,
        &mut super::super::ValueDecodeAdmission<'_>,
    ) -> Result<Value, super::super::ApplicationValueDecodeDenial>,
    marker: PhantomData<fn() -> (Entity, Aspect)>,
}

impl<E, A, T> Clone for RecordField<E, A, T> {
    fn clone(&self) -> Self {
        Self {
            locus: self.locus.clone(),
            path: self.path.clone(),
            recipe: self.recipe.clone(),
            content: self.content.clone(),
            equality: self.equality,
            writable: self.writable,
            presence: self.presence,
            encode: self.encode,
            decode: self.decode,
            marker: PhantomData,
        }
    }
}

/// Selects a typed path without restating the record's field definition.
///
/// Fragment and aspect membership are inferred from the unit marker values.
/// The selected scalar binding belongs to the fragment's binding set.
///
/// Membership is required at the actual field-selection call:
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_schema::{fragment::*, *};
/// struct Suite;
/// struct Item;
/// struct Count;
/// struct Unrelated;
/// struct Row(u64);
/// struct Bindings;
/// impl Fragment for Suite {
///     const NAME: &'static str = "suite";
///     type Bindings = Bindings;
///     type Members = (Item,);
/// }
/// impl Entity for Item {
///     const NAME: &'static str = "item";
///     type Aspects = (Count,);
///     type Relations = ();
/// }
/// impl Aspect for Count { const NAME: &'static str = "count"; type Row = Row; }
/// impl Aspect for Unrelated { const NAME: &'static str = "other"; type Row = Row; }
/// impl BindingSet for Bindings {
///     type Work<'a> = ValueDecodeAdmission<'a>;
///     fn with_admission<T>(work: &mut Self::Work<'_>, f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T) -> T { f(work) }
/// }
/// impl Scalar<u64> for Bindings { type Binding = U64ApplicationValueBinding; }
/// impl Record<Row> for Bindings {
///     fn record() -> impl Shape<Row, Self> {
///         field("value", |row: &Row| row.0).build(|value, _: &mut ValueDecodeAdmission<'_>| Ok::<_, std::convert::Infallible>(Row(value)))
///     }
/// }
/// let _: RecordField<_, _, u64> = record_field(Suite, Item, Unrelated, "value").unwrap();
/// ```
pub fn record_field<F: Fragment, E, A, T: 'static, EntityWitness, AspectWitness>(
    _: F,
    _: E,
    _: A,
    path: &str,
) -> Result<RecordField<E, A, T>, super::super::ApplicationSchemaDeclarationDenial>
where
    E: Entity + In<F::Members, EntityWitness, F>,
    A: Aspect + In<E::Aspects, AspectWitness, E>,
    F::Bindings: Scalar<T> + Record<A::Row>,
{
    checked_record_field::<F, E, A, T>(path)
}

pub(super) fn checked_record_field<F: Fragment, E: Entity, A: Aspect, T: 'static>(
    path: &str,
) -> Result<RecordField<E, A, T>, super::super::ApplicationSchemaDeclarationDenial>
where
    F::Bindings: Scalar<T> + Record<A::Row>,
{
    checked_field::<F::Bindings, E, A, T>(
        path,
        ApplicationFieldBindingLocus::new(E::NAME, super::native_aspect_key::<E, A>(), path),
        &format!("{}.{}.{}", F::NAME, E::NAME, A::NAME),
    )
}
pub(super) fn checked_field<Set, E, A: Aspect, T: 'static>(
    path: &str,
    locus: ApplicationFieldBindingLocus,
    prefix: &str,
) -> Result<RecordField<E, A, T>, super::super::ApplicationSchemaDeclarationDenial>
where
    Set: Scalar<T> + Record<A::Row>,
{
    let shape = <Set as Record<A::Row>>::record();
    let columns = shape.columns("").map_err(|cause| {
        super::super::ApplicationSchemaDeclarationDenial::InvalidRecord {
            path: cause.path().to_owned(),
        }
    })?;
    let column = columns
        .iter()
        .find(|column| column.path == path)
        .ok_or_else(
            || super::super::ApplicationSchemaDeclarationDenial::InvalidRecordField {
                field: locus.clone(),
            },
        )?;
    let actual = (column.recipe)(locus.clone());
    let selected = ApplicationFieldBindingRecipe::of::<<Set as Scalar<T>>::Binding>(locus.clone());
    if actual.value_type() != TypeId::of::<T>() || actual.binding_type() != selected.binding_type()
    {
        return Err(
            super::super::ApplicationSchemaDeclarationDenial::InvalidRecordField { field: locus },
        );
    }
    Ok(RecordField {
        recipe: actual,
        content: column.content.clone(),
        equality: column.equality,
        writable: column.writable,
        presence: column.presence,
        locus,
        path: format!("{prefix}.{path}"),
        encode: |value| {
            <Set as Scalar<T>>::Binding::validate(value)
                .map_err(ApplicationValueEncodeDenial::from)?;
            <Set as Scalar<T>>::Binding::encode(value)
        },
        decode: |value, admission| {
            use super::super::ApplicationReadableScalarValueBinding;
            let decoded = <Set as Scalar<T>>::Binding::decode_admitted(value, admission)?;
            <Set as Scalar<T>>::Binding::validate(&decoded)
                .map_err(super::super::ApplicationValueDecodeDenial::from)?;
            Ok(decoded)
        },
        marker: PhantomData,
    })
}

impl<E, A, T> RecordField<E, A, T> {
    pub fn content_format(&self) -> Option<&super::super::ApplicationContentBindingRecipe> {
        self.content.as_ref()
    }
    pub fn locus(&self) -> &ApplicationFieldBindingLocus {
        &self.locus
    }
    pub fn equality_queryable(&self) -> bool {
        self.equality
    }
    pub fn writable(&self) -> bool {
        self.writable
    }
    pub fn presence(&self) -> super::super::ApplicationFieldPresence {
        self.presence
    }
    pub fn recipe(&self) -> &ApplicationFieldBindingRecipe {
        &self.recipe
    }
    pub fn encode(&self, value: &T) -> Result<ApplicationValue, super::FieldEncodeRefusal> {
        (self.encode)(value)
            .map_err(|cause| super::FieldEncodeRefusal::new(self.path.clone(), cause))
    }
}

impl<S> ApplicationSchemaDeclarationBuilder<S> {
    pub fn operation_write_record_field<O, I, E, A, T: 'static>(
        mut self,
        operation: ApplicationOperationRef<S, O, I>,
        field: RecordField<E, A, T>,
    ) -> Self {
        self.member_provenance
            .fragments
            .expect_field(field.recipe.clone());
        self.push_member_in_place(ApplicationSchemaMember::OperationProgram {
            operation: operation.name().to_owned(),
            target: ApplicationOperationProgramTarget::Write {
                entity: field.locus.entity().to_owned(),
                aspect: field.locus.aspect().to_owned(),
                field: field.locus.field().to_owned(),
            },
        });
        self
    }

    pub fn operation_read_record_field<O, I, E, A, T: 'static>(
        mut self,
        operation: ApplicationOperationRef<S, O, I>,
        field: RecordField<E, A, T>,
    ) -> Self {
        self.member_provenance
            .fragments
            .expect_field(field.recipe.clone());
        self.push_member_in_place(ApplicationSchemaMember::OperationDecisionRead {
            operation: operation.name().to_owned(),
            target: ApplicationOperationDecisionReadTarget::Field {
                entity: field.locus.entity().to_owned(),
                aspect: field.locus.aspect().to_owned(),
                field: field.locus.field().to_owned(),
            },
        });
        self
    }
}

impl<E, A, T: 'static> RecordField<E, A, T> {
    pub fn decode_admitted(
        &self,
        value: &ApplicationValue,
        admission: &mut super::super::ValueDecodeAdmission<'_>,
    ) -> Result<T, super::RecordRefusal> {
        (self.decode)(value, admission).map_err(|cause| {
            super::RecordRefusal::new(&self.path, super::RecordRefusalKind::Decode(cause))
        })
    }
}
