use super::*;
use crate::domain_computation::execution_runtime::product_world::test_product_world_resources;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCreationDenial as Creation;
use crate::facade::primary_graph::WorthQueryBranchAdoptionActivationDenial as Adoption;

#[test]
fn defensive_gate_overflow_and_allocation_refund_the_live_reservation() {
    let limit = test_product_world_resources()
        .budgets()
        .live_product_branches();
    for (reserved, expected) in [
        (
            usize::MAX,
            WorthQueryProductActivationDenial::OccupancyOverflow,
        ),
        (
            usize::MAX - 1,
            WorthQueryProductActivationDenial::AllocationRejected,
        ),
    ] {
        let registry = WorthQueryProductActivationRegistry::new(limit).unwrap();
        let maximum = limit.get().unwrap();
        let held = (1..maximum)
            .map(|_| registry.reserve().unwrap())
            .collect::<Vec<_>>();
        // These secondary counters are bounded by live capacity in consistent
        // operation. Inject desynchronization to reach their defensive producers.
        registry.state.lock().unwrap().reserved_gates = reserved;
        let produced = registry
            .reserve_for_source_program(None)
            .err()
            .expect("the actual registry reservation must be refused");
        assert_eq!(produced, expected);
        let creation = Creation::from(produced);
        assert!(
            matches!(
                (expected, &creation),
                (
                    WorthQueryProductActivationDenial::OccupancyOverflow,
                    Creation::CoordinationOccupancyOverflow
                ) | (
                    WorthQueryProductActivationDenial::AllocationRejected,
                    Creation::CoordinationAllocationRejected
                )
            ),
            "creation must preserve its producer: {creation:?}"
        );
        let adoption = Adoption::from(produced);
        assert!(
            matches!(
                (expected, &adoption),
                (
                    WorthQueryProductActivationDenial::OccupancyOverflow,
                    Adoption::OccupancyOverflow
                ) | (
                    WorthQueryProductActivationDenial::AllocationRejected,
                    Adoption::AllocationRejected
                )
            ),
            "adoption must preserve its producer: {adoption:?}"
        );
        assert_eq!(registry.state.lock().unwrap().reserved_gates, reserved);
        registry.state.lock().unwrap().reserved_gates = held.len();
        let last = registry
            .reserve_for_source_program(None)
            .expect("a defensive map refusal must refund the admitted live slot");
        assert!(
            matches!(registry.reserve(), Err(WorthQueryProductActivationDenial::CapacityExhausted { limit, occupancy }) if (limit, occupancy) == (maximum, maximum))
        );
        drop((last, held));
        assert_eq!(registry.state.lock().unwrap().reserved_gates, 0);
        assert!(registry.live_occurrences().unwrap().is_empty());
    }
}

#[test]
fn unsupported_program_refusals_leave_the_full_activation_limit_available() {
    let revision = program_revision();
    let limit = test_product_world_resources()
        .budgets()
        .live_product_branches();
    let maximum = limit.get().unwrap();
    for program in [None, Some(&revision)] {
        let registry = Arc::new(WorthQueryProductActivationRegistry::new(limit).unwrap());
        registry.require_program_coordination();
        let _retirement = registry.begin_program_retirement(&revision).unwrap();
        for _ in 0..2 {
            assert_eq!(
                registry.reserve_for_source_program(program).err(),
                Some(WorthQueryProductActivationDenial::ProgramSupportUnavailable)
            );
        }
        let held = (0..maximum)
            .map(|_| {
                registry
                    .reserve()
                    .expect("program refusal must leave every live slot free")
            })
            .collect::<Vec<_>>();
        assert!(
            matches!(registry.reserve(), Err(WorthQueryProductActivationDenial::CapacityExhausted { limit, occupancy }) if (limit, occupancy) == (maximum, maximum))
        );
        assert_eq!(
            registry.reserve_for_source_program(program).err(),
            Some(WorthQueryProductActivationDenial::ProgramSupportUnavailable),
            "program support eligibility precedes capacity admission"
        );
        drop(held);
        let state = registry.state.lock().unwrap();
        assert_eq!(state.reserved_gates, 0);
        assert!(state.in_flight_programs.is_empty());
        assert!(state.live_occurrences.is_empty());
    }
}

// Mint a real canonical revision through declaration validation. Program
// contents are irrelevant to this registry's support and reservation lifecycle.
fn program_revision() -> ApplicationProgramRevision {
    use worth_query_declaration::facade::application_program::{
        ApplicationFeatureSpec, ApplicationNoOutputGraph, ApplicationProgramAuthoring,
        ApplicationProgramDefinition, ApplicationProgramIdentity, ApplicationProgramOutputs,
        ApplicationRuleLeaf,
    };
    use worth_query_declaration::facade::application_schema::{
        ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
        ApplicationSchemaDeclarationDenial,
    };

    struct Schema;
    impl ApplicationSchema for Schema {
        const OWNER: &'static str = "worth.query.tests";
        const NAME: &'static str = "activation-capacity";
        const MAJOR: u32 = 1;
        const MINOR: u32 = 0;

        fn declaration(
        ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial>
        {
            ApplicationSchemaDeclarationBuilder::for_schema().build()
        }
    }
    struct Program;
    impl ApplicationProgramDefinition<Schema> for Program {
        type Contributions = ();
        type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
        type Rules = ApplicationRuleLeaf;
        const IDENTITY: ApplicationProgramIdentity =
            ApplicationProgramIdentity::new("worth.query.tests.activation-capacity.v1");

        fn feature_specs() -> Vec<ApplicationFeatureSpec> {
            Vec::new()
        }
    }
    *ApplicationProgramAuthoring::<Schema, Program>::begin()
        .validated_program()
        .expect("registry fixture program is declaration-valid")
        .revision()
}
