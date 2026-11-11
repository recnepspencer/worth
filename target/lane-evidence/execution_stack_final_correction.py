from pathlib import Path
import subprocess,sys,json
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable
results=[]
def lane(label,*cmd):
 code=subprocess.call([py,str(e/'run_evidence.py'),label,*cmd]);results.append([label,code]);(e/'execution-stack-final-correction-results.json').write_text(json.dumps(results,indent=2));return code
code=lane('execution-stack-final-query-lint-driver',py,str(e/'pa_merge_owned_cargo.py'),'execution-stack-final-query-lint','workspaces/worth-query/Cargo.toml','clippy','-p','worth-query-host','--lib','--tests','--keep-going','--','-D','warnings')
if code:raise SystemExit('Read complete final Query diagnostic output')
source=(e/'execution_stack_verify.py').read_text(encoding='utf8');source=source[:source.index("exact_tests('root'")];source=source.replace("out = e / 'execution-stack-verification'","out = e / 'execution-stack-final-transition'")
source+="\nexact_tests('query','workspaces/worth-query/Cargo.toml','worth-query-court-query-decl-host',['host::application_home::program_transition::host_transition_plans_on_nonempty_a_then_serves_writes_closes_and_reopens_b'],'worth_query_court_query_decl_host')\n"
(e/'execution_stack_transition_verify.py').write_text(source,encoding='utf8')
if lane('execution-stack-final-transition-driver',py,str(e/'execution_stack_transition_verify.py')):raise SystemExit('Inspect named transition outcome before other work')
lane('execution-stack-final-query-fmt','cargo','fmt','--manifest-path','workspaces/worth-query/Cargo.toml','-p','worth-query-host','--','--check')
lane('execution-stack-final-line-caps','C:/Program Files/Git/bin/bash.exe','scripts/ci/check_workspace_rust_line_caps.sh','dirty')
lane('execution-stack-final-boundary-driver',py,str(e/'pa_merge_owned_cargo.py'),'execution-stack-final-boundary','tools/boundary-check/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','--root',str(r))
lane('execution-stack-final-context-driver',py,str(e/'pa_merge_owned_cargo.py'),'execution-stack-final-context','tools/agent-context/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','check')
raise SystemExit(int(any(code for _,code in results)))
