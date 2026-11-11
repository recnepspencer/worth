from pathlib import Path
import re,sys,json
sys.path.insert(0,'scripts/quality')
from rust_function_scrutiny import sanitize_rust,signature_terminator,matching_delimiter
E=Path('target/lane-evidence')
for label,counts in [('pa-root-public','4,1,0'),('pa-root-catalog-retained','1,0,0'),('pa-root-native','2,0,0')]:
 s=(E/(label+'.log')).read_text();assert 'EVIDENCE_TEST_COUNTS: '+counts in s,label
 if label!='pa-root-public':assert s.rstrip().endswith('exit_code: 0'),label
plan=(E/'pa-root-consumer-plan.md').read_text();closure=plan.split('## Immutable C2 closure\n',1)[1].split('\nNo ignored',1)[0]
p=Path('plans/database-foundation/milestone-3.md');s=p.read_text();h='### C2 - Cost and bounds: backing and work budgets\n';assert h in s
p.write_text(s.replace(h,h+'\nPinned-reader restoration private remainders:\n'+closure+'\n',1))
p=Path('crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs');s=p.read_text();c=sanitize_rust(s)
n='recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads';m=re.search(r'\bfn\s+'+n+r'\s*\(',c)
a=s.rfind('#[test]',0,m.start());b=matching_delimiter(c,signature_terminator(c,m.end()-1),'{','}')+1
body=s[a:b];assert 'durable_runtime_with_test_schema' not in body
p.write_text('use crate::durability::data::RecoveryVerificationMode as RecoveryMode;\nuse crate::tests::support::*;\n\n'+body+'\n')
(E/'pa-root-retirement-ledger.json').write_text(json.dumps({'base':'5b18804fbd34c814d69aefd527653b11e3d2da59','moved_C2':['native_restore_work_counts_revised_history_sibling_roots_and_partition_images','divergent_sibling_is_not_reused_as_the_storage_mirror'],'qualified_public_only':'shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write','retained_Memory':n,'removed_orphan_family':['shares_first_generation','shares_partition_generation'],'baseline':'unchanged: both private owner files still contain ExecutionRequest roots'},indent=2)+'\n')
print('Retired3qualifiedrootssafely; retained exact Memory catalog assertions; private closure pinned in spec')
