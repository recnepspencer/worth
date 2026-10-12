use super::relational_execution_kind;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
};
use crate::domain_computation::WorthQueryProviderSessionDenialKind as Kind;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

#[test]
fn memory_causes_keep_their_capacity_and_owner_instead_of_becoming_work() {
    for (cause, expected_level, expected_ancestor) in [
        (
            Cause::PolicyMemoryExhausted {
                requested: 17,
                admitted: 11,
                ancestor: 2,
            },
            Level::Policy,
            Some(2),
        ),
        (
            Cause::ProcessMemoryExhausted {
                requested: 17,
                admitted: 11,
            },
            Level::Process,
            None,
        ),
        (
            Cause::DeclaredMemoryExhausted {
                requested: 17,
                admitted: 11,
            },
            Level::Declared,
            None,
        ),
    ] {
        let kind = relational_execution_kind(cause, Some(23)).unwrap();
        let Kind::ExecutionResource {
            denial:
                Resource::MemoryLimit {
                    requested,
                    admitted,
                    level,
                },
            partition_identity,
            policy_ancestor,
        } = kind
        else {
            panic!("memory must retain its typed cause: {kind:?}");
        };
        assert_eq!((requested, admitted), (17, 11));
        assert_eq!(level, expected_level);
        assert_eq!(partition_identity, Some(23));
        assert_eq!(policy_ancestor, expected_ancestor);
    }
    assert!(matches!(
        relational_execution_kind(Cause::WorkExhausted, None),
        Ok(Kind::ExecutionResource {
            denial: Resource::WorkExhausted,
            ..
        })
    ));
}
