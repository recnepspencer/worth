//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct UniqueRecords;
impl fragment::Fragment for UniqueRecords {
    const NAME: &'static str = "unique_records";
    type Bindings = UniqueRecordsBindings;
    type Members = (Member,);
}
pub struct UniqueRecordsBindings;
impl fragment::BindingSet for UniqueRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for UniqueRecordsBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
pub struct Member;
impl fragment::Entity for Member {
    const NAME: &'static str = "Member";
    type Aspects = (MemberProfile,);
    type Relations = ();
}
impl Member {
    pub fn reference() -> ApplicationEntityRef<UniqueSeedSchema, Self> {
        entity(UniqueRecords, Self)
    }
}
pub struct MemberProfile;
impl fragment::Aspect for MemberProfile {
    const NAME: &'static str = "MemberProfile";
    type Row = MemberProfileValue;
}

pub struct MemberProfileValue {
    pub member_handle: u64,
    pub member_tier: u64,
}
impl fragment::Record<MemberProfileValue> for UniqueRecordsBindings {
    fn record() -> impl fragment::Shape<MemberProfileValue, Self> {
        fragment::field("MemberHandle", |row: &MemberProfileValue| row.member_handle)
            .unique()
            .read_only()
            .and("MemberTier", |row: &MemberProfileValue| row.member_tier)
            .build(
                |member_handle, member_tier, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(MemberProfileValue {
                        member_handle,
                        member_tier,
                    })
                },
            )
    }
}
pub type MemberHandle = FieldSelection<UniqueRecords, Member, MemberProfile, u64>;
pub type MemberTier = FieldSelection<UniqueRecords, Member, MemberProfile, u64>;
