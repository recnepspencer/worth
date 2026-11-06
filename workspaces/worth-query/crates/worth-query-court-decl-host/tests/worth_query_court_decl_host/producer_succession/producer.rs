use super::*;
#[derive(Clone, Debug)]
pub enum Denial {
    InitialHasPriorOutput,
    PreserveMissingOutput,
    WrongPriorOutput,
    WrongParent,
}
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "succession.denial" });
worth_query_structured_value_binding!(pub ResultBinding for () { identity: "succession.result" });
pub type Target = WorthQueryInvariantMutationTarget<Schema, Node>;
pub type Scope = ApplicationMutationFieldScope<Key>;
