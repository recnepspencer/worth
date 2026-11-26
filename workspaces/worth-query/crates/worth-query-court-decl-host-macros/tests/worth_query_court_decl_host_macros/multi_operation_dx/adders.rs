//! Ten handler-taking actions sharing one handler type, `Adder`, configured
//! with a different amount per action: a counter moving by exactly that
//! amount proves the builder wired each handler to its own action.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::application_schema::UnitApplicationValueBinding;
use worth_query_decl::facade::WorthQueryAction;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget, WorthQueryOperationBodyDenial,
};

use super::schema::*;

/// Adds `by` to the scoped counter.
pub struct Adder {
    pub by: u64,
}

/// The counter to write and its value after the addition.
pub struct AddDecision {
    pub target: WorthQueryInvariantMutationTarget<BoardSchema, Counter>,
    pub next: u64,
}

type Denial = WorthQueryOperationBodyDenial;

macro_rules! adder_actions {
    ($($Action:ident, $Input:ident, $Op:ident, $Binding:ident, $Done:ident;)+) => {$(
        #[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
        #[worth_query(
            schema = BoardSchema, context = Desk, namespace = BOARD, member,
            input = $Input, operation = $Op, binding = $Binding, delivery = $Done,
            result = UnitApplicationValueBinding, decision = AddDecision, program,
            access(
                read_entities: [Counter];
                read_fields: [CounterFacts::counter_id(), CounterFacts::value()];
                writes: [CounterFacts::value()];
            )
        )]
        pub struct $Action {
            #[worth_query(scope = CounterCounterId)]
            pub counter: u64,
        }

        impl OperationHandler<BoardSchema, $Binding> for Adder {
            fn decide(
                &self,
                input: &$Action,
                reader: &mut DecisionReader<'_, '_, '_, BoardSchema, $Binding>,
            ) -> HandlerResult<AddDecision, Denial> {
                let observed = reader
                    .resolve_entity(CounterFacts::counter_id(), input.counter)
                    .and_then(|counter| {
                        let target = reader.write_target(&counter)?;
                        let value = reader.field(&counter, CounterFacts::value())?;
                        Ok((target, value.unwrap_or_default()))
                    });
                match observed {
                    Ok((target, value)) => HandlerResult::Completed(AddDecision {
                        target,
                        next: value + self.by,
                    }),
                    Err(cause) => HandlerResult::ExecutionDenied(cause),
                }
            }

            fn build_candidate(
                &self,
                _input: &$Action,
                decision: AddDecision,
                writer: &mut CandidateWriter<'_, BoardSchema, $Binding>,
            ) -> HandlerResult<(), Denial> {
                let written = writer.projected_entity(&decision.target).and_then(|counter| {
                    writer.write_field(&counter, CounterFacts::value(), decision.next)
                });
                match written {
                    Ok(()) => HandlerResult::Completed(()),
                    Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
                }
            }
        }
    )+};
}

adder_actions! {
    AddOne, AddOneInput, AddOneOp, AddOneBinding, AddedOne;
    AddTwo, AddTwoInput, AddTwoOp, AddTwoBinding, AddedTwo;
    AddThree, AddThreeInput, AddThreeOp, AddThreeBinding, AddedThree;
    AddFour, AddFourInput, AddFourOp, AddFourBinding, AddedFour;
    AddFive, AddFiveInput, AddFiveOp, AddFiveBinding, AddedFive;
    AddSix, AddSixInput, AddSixOp, AddSixBinding, AddedSix;
    AddSeven, AddSevenInput, AddSevenOp, AddSevenBinding, AddedSeven;
    AddEight, AddEightInput, AddEightOp, AddEightBinding, AddedEight;
    AddNine, AddNineInput, AddNineOp, AddNineBinding, AddedNine;
    AddTen, AddTenInput, AddTenOp, AddTenBinding, AddedTen;
}
