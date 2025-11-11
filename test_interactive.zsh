#!/bin/zsh
# Test script to verify non-blocking prompt behavior
#
# Usage:
#   1. Open a new ZSH session
#   2. cd to slick directory
#   3. Run: SLICK_TEST_DELAY=N source test_interactive.zsh (where N is seconds, e.g., 1, 2, 3)
#   4. Press ENTER to trigger precmd
#   5. You should be able to TYPE IMMEDIATELY (not blocked!)
#   6. After N seconds, the prompt updates with git status

echo "=== Interactive Non-Blocking Test ==="
echo ""
if [[ -n "$SLICK_TEST_DELAY" ]]; then
    echo "Loading slick with ${SLICK_TEST_DELAY}-second delay enabled..."
else
    echo "Loading slick (no delay - set SLICK_TEST_DELAY=N to add delay)..."
fi
echo ""

# Source the loader
source ./load.zsh

echo ""
echo "✓ Slick loaded!"
echo ""
echo "Now press ENTER and start typing immediately"
if [[ -n "$SLICK_TEST_DELAY" ]]; then
    echo "The prompt should NOT block for ${SLICK_TEST_DELAY} seconds!"
    echo ""
    echo "Phase 1 (instant): Shows [user path branch]"
    echo "Phase 2 (${SLICK_TEST_DELAY}s later): Updates with [git status]"
else
    echo "The prompt should render in 2 phases (instant + git status)!"
    echo ""
    echo "Phase 1 (instant): Shows [user path branch]"
    echo "Phase 2 (~50ms later): Updates with [git status]"
fi
echo ""
