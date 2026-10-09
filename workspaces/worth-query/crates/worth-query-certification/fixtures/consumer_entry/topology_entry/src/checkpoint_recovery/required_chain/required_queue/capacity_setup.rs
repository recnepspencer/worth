//! Named initial admissions classify workloads below their subject.
macro_rules! setup_chain {
    ($application:expr, $request:expr) => {{
        use support::capacity_region::{settle as setup_settle, start as setup_start};
        let mut a = setup_start!(
            $request
                .demand(PlanarOutputDemand::new("anchor-a"))
                .start_in_program::<program::ChainProgram, program::ChainRoot>(&$application),
            "register root"
        );
        let mut b = setup_start!(
            $request
                .demand(ChainDemand("anchor-b".to_owned()))
                .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                    &$application
                ),
            "register middle"
        );
        let mut c = setup_start!(
            $request
                .demand(ChainDemand("anchor-c".to_owned()))
                .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                    &$application
                ),
            "register last"
        );
        setup_settle!(a, $request, "initial root");
        setup_settle!(b, $request, "initial middle");
        setup_settle!(c, $request, "initial last");
        (a, b, c)
    }};
}
