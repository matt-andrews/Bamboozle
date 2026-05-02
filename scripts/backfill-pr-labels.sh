#!/bin/bash
# Backfill labels on all PRs based on changed files, mirroring .github/labeler.yml rules.
# Usage: ./scripts/backfill-pr-labels.sh [--dry-run] [--state open|closed|all]

set -euo pipefail

DRY_RUN=false
STATE="all"

while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run) DRY_RUN=true; shift ;;
    --state) STATE="$2"; shift 2 ;;
    *) echo "Unknown argument: $1"; exit 1 ;;
  esac
done

REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)
echo "Repository: $REPO"
$DRY_RUN && echo "DRY RUN — no labels will be applied"
echo ""

# Fetch all PR numbers
pr_numbers=$(gh pr list --repo "$REPO" --state "$STATE" --limit 1000 --json number --jq '.[].number')

if [ -z "$pr_numbers" ]; then
  echo "No PRs found."
  exit 0
fi

total=$(echo "$pr_numbers" | wc -l | tr -d ' ')
echo "Found $total PRs to process"
echo ""

while IFS= read -r pr_num; do
  title=$(gh pr view "$pr_num" --repo "$REPO" --json title --jq '.title' 2>/dev/null || echo "unknown")
  echo "PR #$pr_num: $title"

  files=$(gh pr view "$pr_num" --repo "$REPO" --json files --jq '.files[].path' 2>/dev/null || true)

  if [ -z "$files" ]; then
    echo "  (no files — skipping)"
    continue
  fi

  has_core=false
  has_dotnet=false
  has_npm=false
  has_docs=false
  has_ci=false
  has_examples=false

  while IFS= read -r file; do
    [[ "$file" == bamboozle/* ]]                  && has_core=true
    [[ "$file" == sdks/dotnet/* ]]                && has_dotnet=true
    [[ "$file" == sdks/npm/* ]]                   && has_npm=true
    [[ "$file" == docs/* || "$file" == assets/* || "$file" == README.md || \
       "$file" == DOCKER_HUB_README.md || "$file" == llm.md || "$file" == LICENSE.md ]] \
                                                  && has_docs=true
    [[ "$file" == .github/* || "$file" == playwright/* || "$file" == scripts/* ]] \
                                                  && has_ci=true
    [[ "$file" == examples/* ]]                   && has_examples=true
  done <<< "$files"

  labels=()
  $has_core     && labels+=("area: core")
  $has_dotnet   && labels+=("area: sdks/dotnet")
  $has_npm      && labels+=("area: sdks/npm")
  $has_docs     && labels+=("area: docs")
  $has_ci       && labels+=("area: ci")
  $has_examples && labels+=("area: examples")

  if [ ${#labels[@]} -eq 0 ]; then
    echo "  (no matching labels)"
    continue
  fi

  echo "  labels: ${labels[*]}"

  if ! $DRY_RUN; then
    label_args=()
    for label in "${labels[@]}"; do
      label_args+=(--add-label "$label")
    done
    gh pr edit "$pr_num" --repo "$REPO" "${label_args[@]}"
  fi
done <<< "$pr_numbers"

echo ""
echo "Done."
