//! Standard and content-bearing principals use the same flat storage machinery.
use super::*;
#[derive(Default)]
pub struct ParentRecords<const CONTENT: bool = false>;
impl<const C: bool> Fragment for ParentRecords<C>
where
    PrincipalRecord<C>: Entity,
{
    const NAME: &'static str = "parent_records";
    type Bindings = ParentBindings;
    type Members = (MappingRecord<C>, PrincipalRecord<C>);
}
pub struct ParentBindings;
impl BindingSet for ParentBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for ParentBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for ParentBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
impl Scalar<u64> for ParentBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl Scalar<worth_foundational::facade::ContentRefId> for ParentBindings {
    type Binding = ContentRefApplicationValueBinding;
    const WRITABLE: bool = false;
    fn content_format() -> Option<ApplicationContentBindingRecipe> {
        Some(ApplicationContentBindingRecipe::of::<
            super::super::super::content_binding::codec::Good,
        >())
    }
}
pub struct MappingRecord<const C: bool>;
impl<const C: bool> Entity for MappingRecord<C>
where
    PrincipalRecord<C>: Entity,
{
    const NAME: &'static str = "Mapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTargetRecord<C>,);
}
pub struct PrincipalRecord<const CONTENT: bool>;
impl Entity for PrincipalRecord<false> {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts<false>,);
    type Relations = ();
}
impl Entity for PrincipalRecord<true> {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts<true>,);
    type Relations = ();
}
pub struct MappingTargetRecord<const C: bool>;
impl<const C: bool> Relation for MappingTargetRecord<C>
where
    PrincipalRecord<C>: Entity,
{
    const NAME: &'static str = "MappingTarget";
    type To = PrincipalRecord<C>;
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = MappingRow;
}
pub struct MappingRow {
    identity: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<MappingRow> for ParentBindings {
    fn record() -> impl Shape<MappingRow, Self> {
        field("ExternalIdentityField", |row: &MappingRow| {
            row.identity.clone()
        })
        .equality_queryable(true)
        .and("MappingStatus", |row: &MappingRow| row.status)
        .equality_queryable(true)
        .build(|identity, status, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(MappingRow { identity, status })
        })
    }
}
pub struct PrincipalFacts<const CONTENT: bool = false>;
impl Aspect for PrincipalFacts<false> {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalRow;
}
impl Aspect for PrincipalFacts<true> {
    const NAME: &'static str = "PrincipalFacts";
    type Row = ContentRow;
}
pub struct PrincipalRow(u64);
impl fragment::Record<PrincipalRow> for ParentBindings {
    fn record() -> impl Shape<PrincipalRow, Self> {
        field("PrincipalIdentity", |row: &PrincipalRow| row.0)
            .equality_queryable(true)
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalRow(value))
            })
    }
}
pub struct ContentRow {
    identity: u64,
    samples: worth_foundational::facade::ContentRefId,
}
impl fragment::Record<ContentRow> for ParentBindings {
    fn record() -> impl Shape<ContentRow, Self> {
        field("PrincipalIdentity", |row: &ContentRow| row.identity)
            .equality_queryable(true)
            .and("ContentSamples", |row: &ContentRow| row.samples)
            .build(|identity, samples, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ContentRow { identity, samples })
            })
    }
}
