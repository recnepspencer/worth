from pathlib import Path
import json,os,re,subprocess,sys
root=Path.cwd().resolve()
sys.path.insert(0,str(root/'scripts/test_cost'))
from run_lane import child_environment
from cargo_capture import CargoRunner
from nextest_build import build,list_tests
from coverage import inventory
label=sys.argv[1]
case=sys.argv[2]
out=root/'target/test-cost/query'/label
out.mkdir(parents=True,exist_ok=True)
env=child_environment()
runner=CargoRunner(root,env)
manifest=root/'workspaces/worth-query/Cargo.toml'
reuse=build(manifest,out/'build',runner,selection={'packages':['worth-query-court-query-decl-host'],'targets':{'test':['worth_query_court_query_decl_host']}})
selected=[r for r in inventory(list_tests(reuse,out/'selection.json',runner,'test('+case+')')) if r['selected']]
assert len(selected)==1 and not selected[0]['ignored'],selected
assert selected[0]['test']==case,selected
metadata=json.loads((out/'build/binaries.json').read_text())
row=metadata['rust-binaries'][selected[0]['binary']]
binary=Path(row['binary-path']).resolve()
assert binary.is_relative_to(root)
env.pop('RUST_MIN_STACK',None)
command=[str(binary),'--exact',case,'--nocapture']
print('DIRECT COMMAND:',subprocess.list2cmdline(command),flush=True)
print('RUST_MIN_STACK present:', 'RUST_MIN_STACK' in env,flush=True)
print('BUILD_METADATA:',out/'build/binaries.json',flush=True)
proc=subprocess.Popen(command,cwd=root,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
counts=None
for raw in iter(proc.stdout.readline,b''):
 line=raw.decode('utf-8',errors='replace');print(line,end='',flush=True)
 m=re.search(r'^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',line)
 if m: counts=tuple(map(int,m.groups()))
code=proc.wait()
if counts is None: counts=(0,1,0) if code else (0,0,0)
print('EVIDENCE_TEST_COUNTS: '+','.join(map(str,counts)),flush=True)
print('DIRECT_EXIT_CODE:',code,flush=True)
for path in runner.logs:print('FULL_OWNED_LOG_PATH:',path,flush=True)
sys.exit(code)
