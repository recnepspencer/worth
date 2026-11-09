from pathlib import Path
import json,subprocess,sys,time
r=Path.cwd();e=r/'target/lane-evidence';p=e/'principal-final-focused/results.json'
while True:
 try:done={x['lane'] for x in json.loads(p.read_text())}
 except (OSError,ValueError):done=set()
 if done=={'topology','store','bank'}:break
 time.sleep(1)
print('Portfolio complete; one failed-name charge-attribution rerun, no build',flush=True)
raise SystemExit(subprocess.call([sys.executable,str(e/'run_evidence.py'),'principal-after-measurement-driver',sys.executable,str(e/'principal_after_measure_run.py')]))
