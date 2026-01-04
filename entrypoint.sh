#!/usr/bin/env sh
set -eu

echo ">>> BOOTSTRAP: $(date -Iseconds)"
echo ">>> PWD: $(pwd)"
echo ">>> UID:GID: $(id -u):$(id -g)"
echo ">>> ENV PORT=${PORT:-<unset>}"

if command -v ldd >/dev/null 2>&1; then
  echo ">>> LDD /app/study_ai_planner"
  ldd /app/study_ai_planner || true
else
  echo ">>> LDD not available"
fi

echo ">>> EXEC /app/study_ai_planner"
exec /app/study_ai_planner
