"""Apply source retirement only after both owned counterpart packets pass."""
from pathlib import Path
import json,re,subprocess,sys
sys.path.insert(0,'scripts/quality')
from rust_function_scrutiny import sanitize_rust,signature_terminator,matching_delimiter
E=Path('target/lane-evidence');R=Path('crates/worth-relational/src')
for label,counts in [('pa-next-public','8,0,0'),('pa-next-native','1,0,0')]:
 s=(E/(label+'.log')).read_text(encoding='utf-8');assert 'EVIDENCE_TEST_COUNTS: '+counts in s and s.rstrip().endswith('exit_code: 0'),label
base='f9a53a5d17e01562d503b39b4627252bb63d8639'
removed=[
 'tests/durability/contracts/native_checkpoint/allocation.rs',
 'tests/durability/contracts/native_checkpoint/speculative_symbols.rs',
 'tests/durability/contracts/semantic_result/custom_versions.rs',
 'tests/durability/medium/lineage_frontiers.rs',
]
for file in removed:
 assert subprocess.run(['git','diff','--quiet',base,'--',(R/file).as_posix()]).returncode==0,file
# Record immutable claims in the governing spec before removing any original.
plan=(E/'pa-next-consumer-plan.md').read_text(encoding='utf-8-sig')
closure=plan.split('## Immutable private-claim restore references\n',1)[1].split('\nAfter source review:',1)[0]
c2='\n'.join(line for line in closure.splitlines() if not line.startswith('- C4 full original'))
spec=Path('plans/database-foundation/milestone-3.md');s=spec.read_text(encoding='utf-8')
h='### C2 - Cost and bounds: backing and work budgets\n';assert h in s
s=s.replace(h,h+'\nCapture, dictionary and lineage private remainders:\n'+c2+'\n',1)
h='### C4 - Injected faults\n';assert h in s
negative=next(line for line in closure.splitlines() if line.startswith('- C4 full original'))
shared=next(line for line in closure.splitlines() if line.startswith('Each restore includes'))
s=s.replace(h,h+'\nCustom enforcement decision corruption retains its full original campaign:\n'+negative+'\n\n'+shared+'\n\n',1)
spec.write_text(s,encoding='utf-8')
for file in removed:(R/file).unlink()
for file,before in [
 ('tests/durability/contracts/native_checkpoint.rs','mod allocation;\n'),
 ('tests/durability/contracts/native_checkpoint.rs','mod speculative_symbols;\n'),
 ('tests/durability/contracts/semantic_result.rs','mod custom_versions;\n'),
 ('tests/durability/mod.rs','mod medium;\n'),
]:
 p=R/file;s=p.read_text(encoding='utf-8');assert before in s,(file,before);p.write_text(s.replace(before,''),encoding='utf-8')
p=R/'tests/durability/medium/mod.rs';assert p.read_text(encoding='utf-8').count('mod ')==1;p.unlink();removed.append('tests/durability/medium/mod.rs')
def span(text,name):
 clean=sanitize_rust(text);m=re.search(r'\bfn\s+'+re.escape(name)+r'\s*\(',clean);assert m,name
 a=text.rfind('#[test]',0,m.start());body=signature_terminator(clean,m.end()-1);b=matching_delimiter(clean,body,'{','}')+1
 while b<len(text) and text[b]=='\n':b+=1
 return a,b
p=R/'history/commit/artifact_tests.rs';s=p.read_text(encoding='utf-8')
names=['fresh_scalar_commit_seals_exact_native_lifecycle_and_field_touches','installed_field_index_commit_touch_carries_native_old_and_new_keys']
for a,b in sorted([span(s,n) for n in names],reverse=True):s=s[:a]+s[b:]
s=s.replace('use crate::tests::support::{create_entity_outcome, durable_runtime_with_test_schema};','use crate::tests::support::create_entity_outcome;');p.write_text(s,encoding='utf-8')
p=R/'tests/durability/contracts/semantic_result.rs';s=p.read_text(encoding='utf-8');s=s.replace('use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Artifact;\n','');p.write_text(s,encoding='utf-8')
# identity_frontiers is live in production recovered_counter_capacity; keep it.
p=Path('tools/boundary-check/config/src-court-baseline.json');s=p.read_text(encoding='utf-8')
s='\n'.join(line for line in s.split('\n') if not any((R/f).as_posix() in line for f in removed));p.write_text(s,encoding='utf-8')
(E/'pa-next-retirement-ledger.json').write_text(json.dumps({'base':base,'removed':[(R/f).as_posix() for f in removed],'partial_root':(R/'history/commit/artifact_tests.rs').as_posix(),'partial_tests':names,'native_owner':'memory_checkpoint_preserves_lineage_artifact_and_original_authority','public_cases':json.loads((E/'pa-next-public-cases.json').read_text()),'retained_production_frontiers':'runtime/state/subsystems/lineage.rs remains used by durability/authority/runtime_rebuild/recovered_counter_capacity.rs'},indent=2)+'\n',encoding='utf-8')
print('Retired5source leaves/mounts and2mixed-file roots after8public+1owner green; live production frontiers preserved')
