import datetime,json,os,subprocess,sys
from pathlib import Path
root=Path(__file__).resolve().parents[2]
name, *command=sys.argv[1:]
log=root/'target/lane-evidence'/f'{name}.log'
env=os.environ.copy()
for key in list(env):
 if key.upper()=='RUST_MIN_STACK' or (key.upper().startswith('CARGO_TARGET_') and key.upper().endswith('_LINKER')): env.pop(key)
start=datetime.datetime.now(datetime.timezone.utc).isoformat()
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
clean=not subprocess.check_output(['git','status','--short'],cwd=root,text=True).strip()
with log.open('w',encoding='utf-8') as out:
 out.write(f'COMMAND: {json.dumps(command)}\nHEAD: {head}\nCLEAN: {str(clean).lower()}\nSTART: {start}\nENV: removed stale external linker override; RUST_MIN_STACK unset in parent\n');out.flush()
 try:
  result=subprocess.run(command,cwd=root,env=env,stdout=out,stderr=subprocess.STDOUT)
 except OSError as failure:
  out.write(f'LAUNCH_FAILURE: {failure}\nNo child process was created.\n')
  result=subprocess.CompletedProcess(command,127)
 out.write(f'\nEND: {datetime.datetime.now(datetime.timezone.utc).isoformat()}\nEXIT: {result.returncode}\n')
print(f'{name}: exit={result.returncode} log={log}',flush=True)
sys.exit(result.returncode)
