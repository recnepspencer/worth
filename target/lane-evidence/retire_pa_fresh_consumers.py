from pathlib import Path
import re,sys,json
sys.path.insert(0,'scripts/quality')
from rust_function_scrutiny import sanitize_rust,signature_terminator,matching_delimiter
E=Path('target/lane-evidence')
s=(E/'pa-fresh-public.log').read_text();assert 'EVIDENCE_TEST_COUNTS: 4,1,0' in s
s=(E/'pa-fresh-schema.log').read_text();assert 'EVIDENCE_TEST_COUNTS: 1,0,0' in s and s.rstrip().endswith('exit_code: 0')
plan=(E/'pa-fresh-consumer-plan.md').read_text();closure=plan.split('Full original fresh_source_',1)[1].split('\n\nNo integration',1)[0]
p=Path('plans/database-foundation/milestone-3.md');s=p.read_text();h='### C2 - Cost and bounds: backing and work budgets\n';assert h in s
p.write_text(s.replace(h,h+'\nFresh admission private source/root/schema custody remainder:\nFull original fresh_source_'+closure+'\n\n',1))
R=Path('crates/worth-relational/src')
p=R/'durability/authority/fresh_source/tests.rs';assert p.exists();p.unlink()
p=R/'durability/authority/fresh_source.rs';s=p.read_text();mount='#[cfg(test)]\nmod tests;\n';assert mount in s;p.write_text(s.replace(mount,''))
p=R/'durability/authority/checkpoint_complete/tests/empty_schema.rs';s=p.read_text();c=sanitize_rust(s);n='complete_refuses_recovered_empty_without_original_birth_authority'
m=re.search(r'\bfn\s+'+n+r'\s*\(',c);a=s.rfind('#[test]',0,m.start());b=matching_delimiter(c,signature_terminator(c,m.end()-1),'{','}')+1
s=s[:a]+s[b:];s=s.replace('use crate::durability::data::{\n    RelationalCheckpointArtifactPolicy, RelationalNativeCheckpointCaptureDenial as Denial,\n};','use crate::durability::data::RelationalCheckpointArtifactPolicy;');p.write_text(s)
p=Path('tools/boundary-check/config/src-court-baseline.json');s=p.read_text();removed=(R/'durability/authority/fresh_source/tests.rs').as_posix();p.write_text('\n'.join(line for line in s.split('\n') if removed not in line))
(E/'pa-fresh-retirement-ledger.json').write_text(json.dumps({'base':'5d37a209ee227fdef639cbec2c438d82396a7209','removed_leaf':removed,'removed_partial_test':n,'public_cases':json.loads((E/'pa-fresh-public-cases.json').read_text()),'C2_full_root':'fresh_source_keeps_actual_runtime_root_and_installed_schema_for_complete','baseline':'one deleted leaf row only'},indent=2)+'\n')
print('Retired five qualified roots, complete fresh fixture leaf/mount, orphan Denial import; baseline shrinks only')
