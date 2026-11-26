//! Three features composed as one flat tuple: ten handler actions, the duty
//! actions with the contribution-declared invariant, and the counter read.
use std::time::Duration;

use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::authentication::*;
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_installation::ApplicationBuildIdentity,
    domain::WorthQueryInstalledApplicationSchema, primary_graph::*,
    worth_query_application_contribution, worth_query_application_program,
    WorthQueryApplicationHome, WorthQueryHomeConfig,
};

use super::adders::*;
use super::invariant::{self, CounterCeiling};
use super::schema::*;

worth_query_decl::facade::worth_query_authentication_method!(BoardMethod);
worth_query_decl::facade::worth_query_authentication_audience!(BoardAudience);
worth_query_decl::facade::worth_query_principal_issuer!(BoardIssuer);

macro_rules! leaf_feature {
    ($Feature:ident, $identity:literal) => {
        pub struct $Feature;
        impl ApplicationFeature<BoardSchema> for $Feature {
            type Inputs = ApplicationFeatureInputLeaf;
            const IDENTITY: &'static str = $identity;
        }
    };
}
leaf_feature!(AdderFeature, "multi-operation.dx.adders.v1");
leaf_feature!(DutyFeature, "multi-operation.dx.duty.v1");
leaf_feature!(ReadFeature, "multi-operation.dx.read.v1");

worth_query_application_contribution! {
    pub feature AdderActions<BoardSchema> for AdderFeature => AdderRuntime {
        action one(AddOneBinding, AddedOne, Adder) {};
        action two(AddTwoBinding, AddedTwo, Adder) {};
        action three(AddThreeBinding, AddedThree, Adder) {};
        action four(AddFourBinding, AddedFour, Adder) {};
        action five(AddFiveBinding, AddedFive, Adder) {};
        action six(AddSixBinding, AddedSix, Adder) {};
        action seven(AddSevenBinding, AddedSeven, Adder) {};
        action eight(AddEightBinding, AddedEight, Adder) {};
        action nine(AddNineBinding, AddedNine, Adder) {};
        action ten(AddTenBinding, AddedTen, Adder) {};
    }
}

worth_query_application_contribution! {
    pub feature DutyActions<BoardSchema> for DutyFeature => DutyRuntime {
        body_action(ResetMainBinding, MainReset) { operation: reset_main_operation(); };
        body_action(SetByLabelBinding, LabelSet) { operation: set_by_label_operation(); };
        invariant(CounterCeiling) {
            definition: invariant::definition();
            rule: invariant::resolve;
        };
    }
}

worth_query_application_contribution! {
    pub feature ReadActions<BoardSchema> for ReadFeature => ReadRuntime {
        query(ReadCounterBinding) { definition: ReadCounterQuery::definition(); };
    }
}

worth_query_application_program! {
    pub program BoardProgram for BoardSchema {
        identity: "multi-operation.dx.program.v1",
        rules: ApplicationRuleList<
            ApplicationRuleAt<
                ApplicationLocalRuleRef<BoardSchema, DutyFeature, CounterCeiling>,
                ApplicationCommitBoundary,
            >,
            ApplicationRuleLeaf,
        >,
        contribution: BoardContribution {
            identity: "multi-operation.dx.contribution.v1",
            members: [Records, OwnerBinding],
        },
        feature_definition: (AdderActions, DutyActions, ReadActions),
    }
}

pub type BoardHome = WorthQueryApplicationHome<BoardSchema, BoardProgram>;

/// Seeded counters `(id, label, value)`: two share the label `pair`.
pub const SEEDED: [(u64, &str, u64); 4] = [
    (MAIN_COUNTER, "main", 0),
    (2, "pair", 10),
    (3, "pair", 20),
    (4, "solo", 30),
];

/// Each action's amount equals its position, so `seven` adds 7.
pub fn adders() -> AdderRuntime {
    AdderRuntime::builder()
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
        .expect("every adder action has its handler")
}

fn identity(name: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(BoardIssuer, name).unwrap()
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BoardSchema>,
    installed: &WorthQueryInstalledApplicationSchema<BoardSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = installed
        .principal_binding(OwnerBinding::reference())
        .unwrap();
    let key = WorthQueryApplicationPrincipalKey::new("owner").unwrap();
    let enabled = WorthQueryPrincipalMappingStatus::Enabled;
    graph.bind_principal(&binding, key, 1_u64, identity("owner"), enabled)?;
    for (id, label, value) in SEEDED {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Counter::reference(),
                WorthQueryApplicationEntityKey::new(format!("counter-{id}")).unwrap(),
            )
            .field(CounterFacts::counter_id(), id)
            .field(CounterFacts::label(), label.to_owned())
            .field(CounterFacts::value(), value),
        )?;
    }
    Ok(())
}

/// Opens the board in memory, configured from the flat tuple of feature
/// runtimes.
pub fn open(clock: &WorthQueryAdmissionClock) -> BoardHome {
    let build = ApplicationBuildIdentity::parse("6e0d6e0d-6e0d-4e0d-8e0d-6e0d6e0d6e0d").unwrap();
    let runtimes = (adders(), DutyRuntime::default(), ReadRuntime::default());
    let config = WorthQueryHomeConfig::new(build, (runtimes,))
        .with_clock(clock.clone())
        .with_initial_state(|_, graph, installed| seed(graph, installed));
    WorthQueryApplicationHome::open_memory(config)
        .unwrap_or_else(|denial| panic!("board Home opens: {denial}"))
}

pub fn owner(
    home: &BoardHome,
    clock: &WorthQueryAdmissionClock,
) -> WorthQueryAuthenticatedExternalPrincipal<BoardSchema> {
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    WorthQueryLocalProcessPrincipal::new(
        identity("owner"),
        BoardAudience,
        BoardMethod,
        Duration::from_secs(600),
    )
    .unwrap()
    .admit(home.primary_graph().installed_schema(), clock.clone())
    .expect("local principal admits")
    .authenticate_local(&scope)
    .expect("local principal authenticates")
}
