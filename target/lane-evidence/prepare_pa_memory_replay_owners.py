from pathlib import Path
import re,json
E=Path('target/lane-evidence');D=E/'pa-memory-replay-owner-sources';D.mkdir(exist_ok=True)
R=Path('crates/worth-relational/src')
paths=['authorization/tests/durable_dependencies/recovery.rs','tests/history/replay/lineage_authority_basis.rs','durability/authority/runtime_rebuild/envelope_replay/interruption_tests.rs']
cases=[]
for i,f in enumerate(paths):
 s=(R/f).read_text()
 s=s.replace('crate::tests::support::checkpoint_durably(execution, &runtime)', 'runtime.durability_authority().checkpoint(execution)')
 pattern=r'crate::tests::support::recovery_plan_for\(\s*execution,\s*(&?source|&runtime),\s*(RecoveryVerificationMode::NormalRecoveryVerification),?\s*\)'
 def repl(m):
  receiver=m[1].removeprefix('&');return receiver+'.durability().recovery_plan(execution, '+m[2]+')'
 s=re.sub(pattern,repl,s)
 assert 'recovery_plan_for(' not in s and 'checkpoint_durably(' not in s,f
 (D/str(i)).mkdir(exist_ok=True);(D/str(i)/Path(f).name).write_text(s)
 if i==1:cases.append('audit_replay_uses_checkpoint_owner_basis_when_tail_is_absent')
 else:cases+=re.findall(r'#\[test\]\s*fn (\w+)',s)
(E/'pa-memory-replay-owner-files.json').write_text(json.dumps(paths,indent=2)+'\n')
(E/'pa-memory-replay-owner-cases.json').write_text(json.dumps(cases,indent=2)+'\n')
print('Prepared',len(paths),'actualMemoryownerfiles',len(cases),'affectedcases; notmounted')
