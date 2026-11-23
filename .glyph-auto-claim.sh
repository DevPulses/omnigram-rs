#!/usr/bin/env bash
set -euo pipefail
cd /home/loth/Projects/omnigram-rs
AGENT=glyph
LOG=/home/loth/Projects/omnigram-rs/.glyph-auto-claim.log
TS=$(date -Is)
# Try to claim any deps-satisfied pending task whose related are not locked by others.
# agent-hub next may fail if related busy; fall back to manual scan.
if ./target/release/agent-hub next --agent "$AGENT" >>"$LOG" 2>&1; then
  echo "$TS claimed-via-next" >>"$LOG"
  exit 0
fi
python3 - <<'PY' >>"$LOG" 2>&1
import json, subprocess, os, datetime
AGENT=os.environ.get('AGENT','glyph')
fm=json.load(open('file_map.json'))
st=json.load(open('.agent-state.json'))['tasks']
for t in fm['tasks']:
    tid=t['id']
    s=st.get(tid,{})
    if s.get('status')!='pending': continue
    if not all(st.get(d,{}).get('status')=='done' for d in t.get('depends_on',[])): continue
    # skip if related locked by other agent
    related=t.get('related',[])
    blocked=False
    for r in related:
        rs=st.get(r,{})
        if rs.get('locked') and rs.get('agent') and rs.get('agent')!=AGENT:
            blocked=True; break
    if blocked: continue
    # also skip if self locked by other
    if s.get('locked') and s.get('agent') and s.get('agent')!=AGENT: continue
    r=subprocess.run(['./target/release/agent-hub','claim','--id',tid,'--agent',AGENT], capture_output=True, text=True)
    print(datetime.datetime.now().isoformat(), tid, r.returncode, r.stdout.strip(), r.stderr.strip())
    if r.returncode==0:
        break
PY
