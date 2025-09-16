import json
from pathlib import Path
p = Path('target') / 'geiger' / 'geiger-full.json'
out = Path('target') / 'geiger' / 'extract.txt'
if not p.exists():
    out.write_text(f'MISSING {p}\n')
    raise SystemExit(2)

d = json.loads(p.read_text(encoding='utf-8'))
names = ['getrandom','signal-hook-registry','secp256k1','backtrace','bytes','smallvec']
lines = []
for pkg in d.get('packages', []):
    name = pkg.get('package', {}).get('id', {}).get('name')
    if name in names:
        u = pkg.get('unsafety', {})
        lines.append('---')
        lines.append(name)
        lines.append(f"forbids_unsafe: {u.get('forbids_unsafe')}")
        used = u.get('used', {})
        for k, v in used.items():
            lines.append(f"used.{k}: {v}")

# Ensure we mention missing names
found = {line for line in lines if line and not line.startswith('used') and not line.startswith('forbids') and line != '---'}
for n in names:
    if n not in found:
        lines.append('---')
        lines.append(n)
        lines.append('NOT FOUND')

out.write_text('\n'.join(lines) + '\n')
print('wrote', out)
