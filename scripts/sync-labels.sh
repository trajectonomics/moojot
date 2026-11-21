#!/usr/bin/env bash
# Sync GitHub labels from .github/labels.yml.
# Usage: scripts/sync-labels.sh [--prune] [owner/repo]
#   --prune  delete labels on GitHub that are not in labels.yml
set -euo pipefail

PRUNE=false
if [[ "${1:-}" == "--prune" ]]; then PRUNE=true; shift; fi
REPO="${1:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}"
FILE="$(dirname "$0")/../.github/labels.yml"

mapfile -t ROWS < <(python3 - "$FILE" <<'PY'
import sys, yaml
for l in yaml.safe_load(open(sys.argv[1])):
    print("\t".join([l["name"], l["color"], l.get("description", "")]))
PY
)

declare -A WANT=()
for row in "${ROWS[@]}"; do
  IFS=$'\t' read -r name color desc <<<"$row"
  WANT["$name"]=1
  gh label create "$name" --color "$color" --description "$desc" --force -R "$REPO" >/dev/null
  echo "✓ $name"
done

if $PRUNE; then
  while IFS= read -r existing; do
    [[ -z "$existing" || -n "${WANT[$existing]:-}" ]] && continue
    gh label delete "$existing" --yes -R "$REPO" >/dev/null
    echo "✗ $existing (pruned)"
  done < <(gh label list -R "$REPO" --limit 500 --json name -q '.[].name')
fi
