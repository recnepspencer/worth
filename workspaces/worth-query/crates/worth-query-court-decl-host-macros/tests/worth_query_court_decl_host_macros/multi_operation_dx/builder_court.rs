//! A ten-action feature is built by member name and each handler reaches its
//! own action; a missing handler is refused where the runtime is built.
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

use super::adders::*;
use super::counter;
use super::program::{open, owner, AdderRuntime};

#[test]
fn ten_handlers_built_by_name_each_drive_their_own_action_across_composed_features() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&clock);
    let session = home.session(owner(&home, &clock));

    // Counter 4 starts at 30; action N adds N, so a crossed wire is visible.
    let mut expected = 30;
    macro_rules! add {
        ($Action:ident, $by:literal) => {
            session
                .run($Action { counter: 4 })
                .unwrap_or_else(|denial| panic!("{} runs: {denial:?}", stringify!($Action)));
            expected += $by;
            assert_eq!(
                counter(&home, &clock, 4).value,
                expected,
                "{}",
                stringify!($Action)
            );
        };
    }
    add!(AddOne, 1);
    add!(AddTwo, 2);
    add!(AddThree, 3);
    add!(AddFour, 4);
    add!(AddFive, 5);
    add!(AddSix, 6);
    add!(AddSeven, 7);
    add!(AddEight, 8);
    add!(AddNine, 9);
    add!(AddTen, 10);
    assert_eq!(expected, 85);
    // Only the scoped counter moved.
    assert_eq!(counter(&home, &clock, 2).value, 10);
}

#[test]
fn a_runtime_missing_a_handler_is_refused_by_build_naming_that_action() {
    let partial = AdderRuntime::builder()
        .one(Adder { by: 1 })
        .two(Adder { by: 2 })
        .build();
    let Err(denial) = partial else {
        panic!("eight actions still lack handlers");
    };
    assert_eq!(
        denial,
        WorthQueryPrimaryGraphInstallationDenial::missing_mutation_handler::<
            super::schema::BoardSchema,
            AddThreeBinding,
        >()
    );

    // A later setter replaces an earlier one; supplying all ten builds.
    let complete = AdderRuntime::builder()
        .one(Adder { by: 100 })
        .one(Adder { by: 1 })
        .two(Adder { by: 2 })
        .three(Adder { by: 3 })
        .four(Adder { by: 4 })
        .five(Adder { by: 5 })
        .six(Adder { by: 6 })
        .seven(Adder { by: 7 })
        .eight(Adder { by: 8 })
        .nine(Adder { by: 9 })
        .ten(Adder { by: 10 })
        .build()
        .expect("all ten handlers are present");
    assert_eq!(complete.one.as_ref().map(|adder| adder.by), Some(1));
}
