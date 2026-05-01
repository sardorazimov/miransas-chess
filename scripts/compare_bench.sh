#!/usr/bin/env bash
# Usage: compare_bench.sh <main.txt> <pr.txt>
# Prints a markdown table comparing NPS between main and PR bench runs.

set -euo pipefail

MAIN_FILE="${1:-main.txt}"
PR_FILE="${2:-pr.txt}"

echo "| Metric | main NPS | PR NPS | Δ% |"
echo "|--------|----------|--------|-----|"

if grep -q "MAIN HAS NO BENCH BINARY" "$MAIN_FILE" 2>/dev/null; then
    echo "| _(no baseline on main)_ | — | — | — |"
    echo ""
    echo "Note: NPS on shared CI runners has ±15% noise — treat <5% changes as inconclusive."
    exit 0
fi

while IFS= read -r main_line; do
    if [[ "$main_line" =~ ^(PERFT|SEARCH)[[:space:]]([^[:space:]]+)[[:space:]] ]]; then
        kind="${BASH_REMATCH[1]}"
        name="${BASH_REMATCH[2]}"
        key="$kind $name"

        pr_line=$(grep "^$key " "$PR_FILE" 2>/dev/null || true)
        if [[ -z "$pr_line" ]]; then
            continue
        fi

        main_nps=$(echo "$main_line" | grep -oP 'nps=\K[0-9]+' || echo "0")
        pr_nps=$(echo "$pr_line" | grep -oP 'nps=\K[0-9]+' || echo "0")

        if [[ "$main_nps" == "inf" || "$pr_nps" == "inf" || "$main_nps" -eq 0 ]]; then
            delta="N/A"
        else
            delta=$(awk "BEGIN { printf \"%.1f\", ($pr_nps - $main_nps) * 100.0 / $main_nps }")
        fi

        echo "| $kind $name | $main_nps | $pr_nps | ${delta}% |"
    fi
done < "$MAIN_FILE"

echo ""
echo "Note: NPS on shared CI runners has ±15% noise — treat <5% changes as inconclusive."
