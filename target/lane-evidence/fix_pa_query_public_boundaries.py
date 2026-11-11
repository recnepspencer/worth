from pathlib import Path
import json
E=Path('target/lane-evidence');D=Path('crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication')
# The old generic index parity implementation is cfg(test), not a product capability.
# Preserve its unqualified candidates and untouched Native roots for a separate owner split.
p=D/'checkpoint_query_parity.rs';assert p.exists();(E/'pa-query-recovery-sources/checkpoint_query_parity.rs').write_text(p.read_text());p.unlink()
p=D/'mod.rs';s=p.read_text();assert '\nmod checkpoint_query_parity;\n' in s;p.write_text(s.replace('\nmod checkpoint_query_parity;\n','\n'))
p=D/'checkpoint_truth_equivalence.rs';s=p.read_text()
s=s.replace('lineage::{HistoricalResolutionBoundednessBasis, HistoricalResolutionRequest},\n','')
a=s.index('        let target = HistoricalResolutionRequest {');b=s.index('        assert_eq!(',a);s=s[:a]+s[b:]
s=s.replace('.lineage_access()\n                .entity_aspect_history_with_trace(execution, target.clone(), None)', '.history()\n                .lineage_entity_aspect_history_with_trace(execution, &main, *lineage, None)')
s=s.replace('.lineage_access()\n                .entity_aspect_history_with_trace(execution, target, None)', '.history()\n                .lineage_entity_aspect_history_with_trace(execution, &main, *lineage, None)')
assert 'lineage_access' not in s and 'HistoricalResolution' not in s;p.write_text(s)
p=D/'checkpoint_topology_reidentification.rs';s=p.read_text()
s=s.replace('HistoricalResolutionDigestMode, HistoricalResolutionRequest,','HistoricalResolutionDigestMode,')
for subject in ['entity','replaced_entity']:
 runtime='runtime'
 before=f'''runtime
        .lineage_access()
        .for_record(execution, {subject})
        .unwrap()
        .unwrap()
        .lineage_id'''
 assert before in s,subject
 after=f'''runtime
        .lineage_node_at_observation(execution, &runtime.observe_branch(&runtime.main_branch_identity()).unwrap().1.observation(), {subject})
        .unwrap().unwrap().node().lineage_id()'''
 s=s.replace(before,after)
for runtime in ['runtime','recovered']:
 before=f'''{runtime}
        .lineage_access()
        .resolve_historical_lineage(
            execution,
            HistoricalResolutionRequest {{
                branch_id: BranchId("main".to_string()),
                lineage_id: start_lineage,
                boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            }},
        )'''
 assert before in s,runtime
 after=f'''{runtime}.discover_authored_lineage_history(execution, &{runtime}.main_branch_identity(), start_lineage)'''
 s=s.replace(before,after)
before='''runtime
        .lineage_access()
        .entity_aspect_history(
            execution,
            HistoricalResolutionRequest {
                branch_id: BranchId("main".to_string()),
                lineage_id: start_lineage,
                boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            },
            None,
        )'''
assert before in s
s=s.replace(before,'''runtime.history().lineage_entity_aspect_history(execution, &BranchId("main".into()), start_lineage, None)''')
anchor='    assert!(!resolution.resolved.is_empty());';assert anchor in s
s=s.replace(anchor,'''    assert_eq!(resolution.digest_basis().boundedness_basis(), HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed);
'''+anchor)
assert 'lineage_access' not in s and 'HistoricalResolutionRequest {' not in s;p.write_text(s)
cases=json.loads((E/'pa-query-recovery-cases.json').read_text());cases=[n for n in cases if not n.startswith('derived_index_contract_')]
assert len(cases)==2;(E/'pa-query-recovery-cases.json').write_text(json.dumps(cases,indent=2)+'\n')
print('Corrected all7lineage calls to existing shipping owners; preserved2unqualifiedlegacyparitycandidates andoriginalNative roots')
