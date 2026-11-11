from pathlib import Path
import json
E=Path('target/lane-evidence');R=Path('crates/worth-relational/src')
s=(R/'tests/domains/cad/topology_identity_survival.rs').read_text();s=s[s.index('#[test]'):]
s=s.replace('crate::test_execution_policy::serial_request()', 'worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request()')
s=s.replace('let runtime = durable_runtime_with_test_schema();','let medium = TestDurableMedium::new();\n    let runtime = durable_runtime(&medium);')
s=s.replace('changed_entities(&created)[0]','worth_testkit_relational::aspect_storage::publications::changed_entities(&created)[0]')
s=s.replace('changed_entities(&replacement)[0]','worth_testkit_relational::aspect_storage::publications::changed_entities(&replacement)[0]')
s=s.replace('update_entity(execution, &runtime, entity, "topology-source-updated")','super::checkpoint_index_generations::update_entity_on_branch(execution, &runtime, entity, "topology-source-updated", BranchId("main".into()))')
a=s.index('    crate::tests::support::checkpoint_durably');b=s.index('    let recovered_resolution',a)
s=s[:a]+'    medium.file_checkpoint(capture(execution, &runtime));\n    let (recovered, _) = recover(execution, &medium);\n\n'+s[b:]
s=s.replace('assert_recovered_commit_truth_matches(', 'assert_recovered_commit_truth(')
s=s.replace('        &[replaced_entity],\n        &[],\n        &[start_lineage, replacement_lineage],','        TruthSubjects { entities: &[replaced_entity], relations: &[], lineages: &[start_lineage, replacement_lineage] },')
h='''//! Original topology reidentification and lineage histories survive a real source.
use super::*;
use super::checkpoint_truth_equivalence::{assert_recovered_commit_truth, TruthSubjects};
use worth_relational::facade::{history::{BranchId, AspectHistoryEntry, AspectResolutionContext}, identity::LineageId, lineage::{HistoricalResolutionBoundednessBasis, HistoricalResolutionDigestMode, HistoricalResolutionRequest}};
'''
origin=(R/'tests/support.rs').read_text();a=origin.index('pub(super) fn assert_lineage_history_origin_invariants');b=origin.index('\n#[path',a)
helper=origin[a:b].replace('pub(super) fn','fn')
(E/'pa-query-recovery-sources/checkpoint_topology_reidentification.rs').write_text(h+s+'\n'+helper)
p=E/'pa-query-recovery-cases.json';cases=json.loads(p.read_text());n='topology_identity_survival_preserves_reidentification_truth_across_recovery'
if n not in cases:cases.append(n)
p.write_text(json.dumps(cases,indent=2)+'\n')
print('Prepared original topology case and exact public truth-family counterpart; not mounted')
