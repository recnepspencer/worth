use crate::{
    application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema,
};

/// How a displayed optional member differs from its initialized value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationOptionalMemberEdit<Member> {
    Unchanged,
    Set(Member),
    Clear,
}

/// Typed projection between one repeated result row and one program action.
///
/// Implementations define product meaning only. An installed program must
/// explicitly attach the correspondence before a client can obtain its
/// projection handle.
pub trait ApplicationRepeatedOptionalMemberCorrespondence<Schema, Binding>: 'static
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    type Row;
    type Target: Clone;
    type Member: Clone;

    const IDENTITY: &'static str;

    fn target(row: &Self::Row) -> Self::Target;
    fn initial_member(row: &Self::Row) -> Option<Self::Member>;
    fn input(target: &Self::Target, member: Option<Self::Member>) -> Binding::Input;
}
