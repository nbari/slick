#!/bin/bash
# Test timeout protection by trying to fetch from a hanging remote

echo "Testing timeout protection..."
echo "This should complete in ~15 seconds max (not hang forever)"
echo ""

# Create a test repo with a fake remote that will timeout
TMPDIR=$(mktemp -d)
cd "$TMPDIR" || exit 1

git init
git config user.email "test@example.com"
git config user.name "Test"

# Add a remote that will timeout (non-routable IP)
git remote add origin ssh://git@192.0.2.1/fake/repo.git

echo "test" > file.txt
git add file.txt
git commit -m "test"

echo "Running slick precmd with SLICK_PROMPT_GIT_FETCH=1..."
echo "If this hangs, the timeout protection failed!"
echo ""

start=$(date +%s)
SLICK_PROMPT_GIT_FETCH=1 timeout 20 slick precmd
exit_code=$?
end=$(date +%s)
elapsed=$((end - start))

echo ""
echo "Completed in ${elapsed} seconds"

if [ $exit_code -eq 124 ]; then
    echo "❌ FAILED: Command was killed by timeout (took > 20s)"
    exit 1
elif [ $elapsed -gt 18 ]; then
    echo "⚠️  WARNING: Took longer than expected (${elapsed}s > 18s)"
    exit 1
else
    echo "✅ SUCCESS: Completed quickly despite unreachable remote"
fi

# Cleanup
cd /
rm -rf "$TMPDIR"
