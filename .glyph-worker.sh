#!/usr/bin/env bash
set -u
cd /home/loth/Projects/omnigram-rs
AGENT=glyph
LOG=/home/loth/Projects/omnigram-rs/.glyph-worker.log
INTERVAL=300
while true; do
  TS=$(date -Is)
  echo "--- $TS start ---" >>"$LOG"
  # Check if there is any pending task we can claim
  python3 - "$AGENT" >>"$LOG" 2>&1 <<'PY'
import json, subprocess, sys, os, datetime
AGENT=sys.argv[1] if len(sys.argv)>1 else 'glyph'
try:
  fm=json.load(open('file_map.json'))
  st=json.load(open('.agent-state.json'))
except Exception as e:
  print("ERR read:", e); sys.exit(1)
tasks=st.get('tasks',{})
for t in fm['tasks']:
    tid=t['id']
    s=tasks.get(tid)
    # if not in state, skip claiming logic by hub for now (hub may reject)
    if not s:
        continue
    if s.get('status')!='pending':
        continue
    if s.get('locked'):
        continue
    # deps must be done
    deps=t.get('depends_on',[])
    if not all(tasks.get(d,{}).get('status')=='done' for d in deps):
        continue
    # related must not be locked by others
    related=t.get('related',[])
    blocked=False
    for r in related:
        rs=tasks.get(r,{})
        if rs.get('locked') and rs.get('agent') and rs.get('agent')!=AGENT:
            blocked=True; break
    if blocked:
        continue
    print(f"TRY claim {tid}")
    r=subprocess.run(['./target/release/agent-hub','claim','--id',tid,'--agent',AGENT], capture_output=True, text=True)
    print('RET', r.returncode, r.stdout.strip(), r.stderr.strip())
    if r.returncode==0:
        print('CLAIMED', tid)
        sys.exit(10)  # signal claimed
print('NONE')
PY
  RC=$?
  echo "--- $TS rc=$RC ---" >>"$LOG"
  if [ $RC -eq 10 ]; then
    echo "$TS claimed a task, exiting worker loop for manual work." >>"$LOG"
    break
  fi
  sleep "$INTERVAL"
done
