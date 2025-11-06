#!/bin/zsh
# Test script to verify non-blocking prompt behavior
#
# Usage:
#   1. Open a new ZSH session
#   2. cd to slick directory
#   3. Run: SLICK_TEST_DELAY=1 source test_interactive.zsh
#   4. Press ENTER to trigger precmd
#   5. You should be able to TYPE IMMEDIATELY (not blocked!)
#   6. After 3 seconds, the prompt updates with git status

echo "=== Interactive Non-Blocking Test ==="
echo ""
echo "Loading slick with 3-second delay enabled..."
echo ""

# Source the loader
source ./load.zsh

echo ""
echo "✓ Slick loaded!"
echo ""
echo "Now press ENTER and start typing immediately"
echo "The prompt should NOT block for 3 seconds!"
echo ""
echo "Phase 1 (instant): Shows [user path branch]"
echo "Phase 2 (3s later): Updates with [git status]"
echo ""
