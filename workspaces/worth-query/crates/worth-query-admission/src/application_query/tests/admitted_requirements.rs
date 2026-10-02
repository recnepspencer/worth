use super::{
    account_parameter, admitted_requirements, installed_query, test_canonical_budget,
    ApplicationQueryParameterSet, WorthQueryApplicationQueryLane,
};
use crate::application_query::requirements::derive_graph_read_access_requirements_for_contract_admitted;
use crate::canonical_identity_derivation::WorthQueryCanonicalIdentityStop;
use crate::facade::application_query::admit_application_query_parameters;

#[derive(Debug, Eq, PartialEq)]
enum Refusal {
    Work,
    Bytes,
    BeforeInventory,
}

#[test]
fn installed_contract_admitted_requirements_preserve_ordinary_meaning() {
    let query = installed_query();
    for parameter in [7_u64, 8] {
        let parameters = admit_application_query_parameters(
            &query,
            ApplicationQueryParameterSet::new()
                .bind(account_parameter(), parameter)
                .unwrap(),
        )
        .unwrap();
        for lane in [
            WorthQueryApplicationQueryLane::OneShot,
            WorthQueryApplicationQueryLane::Continuation,
            WorthQueryApplicationQueryLane::Live,
        ] {
            for maximum in [1, 32] {
                let ordinary =
                    admitted_requirements(query.read_graph(), lane, maximum, parameters.identity());
                let mut claims = Vec::new();
                let actual = derive_graph_read_access_requirements_for_contract_admitted(
                    query.read_family_binding().planning_contract(),
                    lane,
                    maximum,
                    parameters.identity(),
                    test_canonical_budget(),
                    &mut |work, bytes| {
                        claims.push((work, bytes));
                        Ok::<_, Refusal>(())
                    },
                )
                .unwrap();
                // Equality includes every row, digest, counter and canonical
                // evidence field, rather than only the rendered identity.
                assert_eq!(ordinary, actual);
                assert!(claims.iter().any(|(_, bytes)| *bytes > 0));
                assert!(claims.iter().any(|(work, _)| *work > 0));
            }
        }
    }
}

#[test]
fn installed_contract_refusal_preserves_the_resource_cause() {
    let query = installed_query();
    let parameters = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 7_u64)
            .unwrap(),
    )
    .unwrap();
    let derive = |admit: &mut dyn FnMut(u64, u64) -> Result<(), Refusal>| {
        derive_graph_read_access_requirements_for_contract_admitted(
            query.read_family_binding().planning_contract(),
            WorthQueryApplicationQueryLane::OneShot,
            32,
            parameters.identity(),
            test_canonical_budget(),
            &mut |work, bytes| admit(work, bytes),
        )
    };
    let mut claims = Vec::new();
    derive(&mut |work, bytes| {
        claims.push((work, bytes));
        Ok(())
    })
    .unwrap();
    let total_work: u64 = claims.iter().map(|(work, _)| work).sum();
    let total_bytes: u64 = claims.iter().map(|(_, bytes)| bytes).sum();
    assert!(total_work > 1 && total_bytes > 1);

    for (work_limit, byte_limit, expected) in [
        (total_work - 1, total_bytes, Refusal::Work),
        (total_work, total_bytes - 1, Refusal::Bytes),
    ] {
        let mut consumed = (0_u64, 0_u64);
        let mut refused = false;
        let stopped = derive(&mut |work, bytes| {
            assert!(!refused, "preparation continued after admission refused");
            if work > work_limit - consumed.0 {
                refused = true;
                return Err(Refusal::Work);
            }
            if bytes > byte_limit - consumed.1 {
                refused = true;
                return Err(Refusal::Bytes);
            }
            consumed.0 += work;
            consumed.1 += bytes;
            Ok(())
        });
        assert!(
            matches!(stopped, Err(WorthQueryCanonicalIdentityStop::Admission(cause)) if cause == expected)
        );
        assert!(refused);
    }

    let mut calls = 0;
    let stopped = derive(&mut |_, _| {
        calls += 1;
        Err(Refusal::BeforeInventory)
    });
    assert!(matches!(
        stopped,
        Err(WorthQueryCanonicalIdentityStop::Admission(
            Refusal::BeforeInventory
        ))
    ));
    assert_eq!(calls, 1);
}
