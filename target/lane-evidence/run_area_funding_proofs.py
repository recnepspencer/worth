from pathlib import Path
import subprocess,json,sys,shutil
for lane,*command in json.loads(Path(sys.argv[1]).read_text()):
 free=shutil.disk_usage(Path.cwd().anchor).free
 print('START',lane,'free_GiB',round(free/(1<<30),1),flush=True)
 if free<25*(1<<30): raise SystemExit('Disk safety floor reached')
 result=subprocess.run([sys.executable,'target/lane-evidence/capture_lane.py',lane,*command])
 print('END',lane,'exit',result.returncode,flush=True)
 if result.returncode: raise SystemExit(result.returncode)
