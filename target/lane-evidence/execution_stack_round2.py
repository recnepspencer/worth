from pathlib import Path
import json,subprocess,sys
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable
results={}
for label,manifest,packages in [('root','Cargo.toml',['worth-runtime-bridge']),('query','workspaces/worth-query/Cargo.toml',['worth-query-execution','worth-query'])]:
 command=[py,str(e/'run_evidence.py'),'execution-stack-round2-'+label+'-check-driver',py,str(e/'pa_merge_owned_cargo.py'),'execution-stack-round2-'+label+'-check',manifest,'check',*[a for p in packages for a in ('-p',p)],'--tests','--message-format=json','--keep-going']
 results[label]=subprocess.call(command)
 (e/'execution-stack-round2-check-results.json').write_text(json.dumps(results,indent=2))
 if results[label]:raise SystemExit('Read all diagnostics before another check')
source=(e/'execution_stack_verify.py').read_text(encoding='utf8')
start=source.index("exact_tests('root'")
end=source.index('# All four genuine mains')
source=source[:start]+source[end:]
source=source.replace("out = e / 'execution-stack-verification'", "out = e / 'execution-stack-round2-verification'")
# These genuine mains use the changed lifecycle cleanup as well, so verify all
# after one shared consumer rebuild. No repeat of unchanged helper/wasm checks.
(e/'execution_stack_round2_verify.py').write_text(source,encoding='utf8')
raise SystemExit(subprocess.call([py,str(e/'run_evidence.py'),'execution-stack-round2-focused-driver',py,str(e/'execution_stack_round2_verify.py')]))
