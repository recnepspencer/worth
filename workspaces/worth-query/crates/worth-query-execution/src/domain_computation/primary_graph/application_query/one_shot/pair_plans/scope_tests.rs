//! Constructor type contracts; no admitted plan or Current value is fabricated.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    Account, Activity, IdentityExecutionSchema,
};

// These query/parameter/result types stay generic: only the two real declared
// scope families are fixed here. The nested consumer court separately executes
// installed plans and checks native identity, basis and batch custody.
fn accepts_distinct_scope_families<
    'a,
    FQ: 'a,
    FP: 'a,
    FR: 'a,
    SQ: 'a,
    SP: 'a,
    SR: 'a,
    A: 'a,
    I: 'a,
>() {
    let _: fn(
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            IdentityExecutionSchema,
            FQ,
            FP,
            FR,
            A,
            I,
            Account,
        >,
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            IdentityExecutionSchema,
            SQ,
            SP,
            SR,
            A,
            I,
            Activity,
        >,
    ) -> WorthQueryDerivedPairReadPlans<
        'a,
        IdentityExecutionSchema,
        FQ,
        FP,
        FR,
        SQ,
        SP,
        SR,
        A,
        I,
        Account,
        Activity,
    > = WorthQueryDerivedPairReadPlans::new;
    let _: fn(
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            IdentityExecutionSchema,
            FQ,
            FP,
            FR,
            A,
            I,
            Account,
        >,
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            IdentityExecutionSchema,
            SQ,
            SP,
            SR,
            A,
            I,
            Activity,
        >,
        &WorthQueryApplicationQueryBatchAdmission,
    ) -> WorthQueryDerivedPairReadPlans<
        'a,
        IdentityExecutionSchema,
        FQ,
        FP,
        FR,
        SQ,
        SP,
        SR,
        A,
        I,
        Account,
        Activity,
    > = WorthQueryDerivedPairReadPlans::in_batch;
}

#[test]
fn closed_pair_constructors_preserve_distinct_declared_scope_families() {
    accepts_distinct_scope_families::<(), (), (), (), (), (), (), ()>();
}
