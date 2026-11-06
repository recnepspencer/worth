//! One record model owns field lowering and reusable scalar bindings.

use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub(super) struct LifecycleRecords;
impl fragment::Fragment for LifecycleRecords {
    const NAME: &'static str = "lifecycle_records";
    type Bindings = LifecycleRecordsBindings;
    type Members = (Parent, Consequence);
}
pub(super) struct LifecycleRecordsBindings;
impl fragment::BindingSet for LifecycleRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for LifecycleRecordsBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<ApplicationSigningSupersession> for LifecycleRecordsBindings {
    type Binding = ApplicationSigningSupersessionBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
pub(super) struct Parent;
impl fragment::Entity for Parent {
    const NAME: &'static str = "Parent";
    type Aspects = (ParentFacts,);
    type Relations = ();
}

pub(super) struct Consequence;
impl fragment::Entity for Consequence {
    const NAME: &'static str = "Consequence";
    type Aspects = (ConsequenceFacts,);
    type Relations = ();
}

pub(super) struct ParentFacts;
impl fragment::Aspect for ParentFacts {
    const NAME: &'static str = "ParentFacts";
    type Row = ParentFactsValue;
}

pub(super) struct ParentFactsValue {
    pub revision: u64,
}
impl fragment::Record<ParentFactsValue> for LifecycleRecordsBindings {
    fn record() -> impl fragment::Shape<ParentFactsValue, Self> {
        fragment::field("Revision", |row: &ParentFactsValue| row.revision).build(
            |revision, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ParentFactsValue { revision })
            },
        )
    }
}
pub(super) struct ConsequenceFacts;
impl fragment::Aspect for ConsequenceFacts {
    const NAME: &'static str = "ConsequenceFacts";
    type Row = ConsequenceFactsValue;
}

pub(super) struct ConsequenceFactsValue {
    pub record: ApplicationSigningSupersession,
}
impl fragment::Record<ConsequenceFactsValue> for LifecycleRecordsBindings {
    fn record() -> impl fragment::Shape<ConsequenceFactsValue, Self> {
        fragment::field("Record", |row: &ConsequenceFactsValue| row.record.clone()).build(
            |record, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ConsequenceFactsValue { record })
            },
        )
    }
}
pub(super) type Revision = FieldSelection<LifecycleRecords, Parent, ParentFacts, u64>;
pub(super) type Record =
    FieldSelection<LifecycleRecords, Consequence, ConsequenceFacts, ApplicationSigningSupersession>;
