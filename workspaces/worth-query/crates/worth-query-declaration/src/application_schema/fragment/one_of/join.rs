//! Joining case lists preserves one flat roster, decoder and group contract.
use super::{ApplicationValue, RecordRefusal};
use super::{BindingSet, Builder, CaseDefinition, CaseRows, Cases, Denial, Lowering};
use std::{collections::BTreeMap, marker::PhantomData};

/// Concatenate two case lists. Either side may itself be another `JoinCases`.
/// Tuple leaves keep their usual arities; composition has no SDK case-count cap.
pub struct JoinCases<Left, Right>(PhantomData<(Left, Right)>);

impl<Set, Enum, Left, Right> CaseRows<Set, Enum> for JoinCases<Left, Right>
where
    Set: BindingSet,
    Left: CaseRows<Set, Enum>,
    Right: CaseRows<Set, Enum>,
{
    fn contains(name: &str) -> bool {
        Left::contains(name) || Right::contains(name)
    }
    fn decode(
        name: &str,
        values: &BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut Set::Work<'_>,
        admission: &mut crate::application_schema::ValueDecodeAdmission<'_>,
    ) -> Result<Enum, RecordRefusal> {
        // Dispatch on declared membership. A selected row's failure must never
        // be mistaken for an invitation to try a different case list.
        if Left::contains(name) {
            Left::decode(name, values, prefix, work, admission)
        } else {
            Right::decode(name, values, prefix, work, admission)
        }
    }
}
impl<S, Set, Enum, Left, Right> Cases<S, Set, Enum> for JoinCases<Left, Right>
where
    Set: BindingSet,
    Left: Cases<S, Set, Enum>,
    Right: Cases<S, Set, Enum>,
{
    fn definitions() -> Vec<CaseDefinition> {
        let mut cases = Left::definitions();
        cases.extend(Right::definitions());
        cases
    }
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
        contract: &[u8; 32],
    ) -> Result<Builder<S>, Denial> {
        let builder = Left::lower(builder, prefix, entity, state, contract)?;
        Right::lower(builder, prefix, entity, state, contract)
    }
}
