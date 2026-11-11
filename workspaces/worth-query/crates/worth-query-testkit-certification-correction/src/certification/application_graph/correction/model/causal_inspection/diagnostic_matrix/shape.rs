//! Typed output slots shared by the four policy declarations.
use super::*;
pub struct OperandSlot;
pub struct CalculatedSlot;
pub struct ReferencesSlot;
pub struct OrderingSlot;
worth_query_portable_type!(OrderingSlot => "diagnostic-matrix.ordering.v1");
worth_query_portable_type!(OperandSlot => "diagnostic-matrix.operand.v1");
worth_query_portable_type!(CalculatedSlot => "diagnostic-matrix.calculated.v1");
worth_query_portable_type!(ReferencesSlot => "diagnostic-matrix.references.v1");
pub(super) fn operand<Q>(
) -> ApplicationQueryOptionalResultFieldRef<Q, OperandSlot, NumericSchema, Operand> {
    ApplicationQueryOptionalResultFieldRef::new("operand", Operand::at("Operand"))
}
pub(super) fn calculated<Q>(
) -> ApplicationQueryResultFieldRef<Q, CalculatedSlot, NumericSchema, Calculated> {
    ApplicationQueryResultFieldRef::new("calculated", Calculated::at("Calculated"))
}
pub(super) fn ordering<Q>(
) -> ApplicationQueryResultFieldRef<Q, OrderingSlot, NumericSchema, Primary> {
    ApplicationQueryResultFieldRef::new("ordering", Primary::at("Primary"))
}
pub(super) fn references<Q>() -> ApplicationQueryResultRelationRef<
    Q,
    ReferencesSlot,
    NumericSchema,
    AuthoredReference,
    Number,
    Number,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("references", AuthoredReference::reference())
}
