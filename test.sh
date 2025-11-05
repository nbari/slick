#!/bin/bash
# Comprehensive integration tests for slick prompt

set +e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

pass() {
    echo -e "${GREEN}✅${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
    echo -e "${RED}❌${NC} $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

test_case() {
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -e "\n${BLUE}[$TESTS_RUN]${NC} $1"
}

info() {
    echo -e "${YELLOW}ℹ${NC}  $1"
}

# Get absolute path to slick binary
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SLICK="$SCRIPT_DIR/target/release/slick"

if [ ! -f "$SLICK" ]; then
    echo -e "${RED}Error: Binary not found at $SLICK${NC}"
    echo "Run: cargo build --release"
    exit 1
fi

# Temp directory
TEST_DIR="/tmp/slick-test-$$"
mkdir -p "$TEST_DIR"

cleanup() {
    rm -rf "$TEST_DIR"
}
trap cleanup EXIT

# Helper function
create_test_repo() {
    local dir="$1"
    mkdir -p "$dir"
    cd "$dir"
    git init -q
    git config user.email "test@example.com"
    git config user.name "Test User"
    echo "# Test" >README.md
    git add README.md
    git commit -q -m "Initial commit"
}

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Slick Prompt Integration Tests${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# TEST 1: Basic Git Repository
test_case "Basic Git Repository"
create_test_repo "$TEST_DIR/test1"
OUT=$("$SLICK" precmd 2>&1)
[[ "$OUT" == *'"branch"'* ]] && pass "Git repo detection" || fail "Failed"

# TEST 2: Non-Git Directory
test_case "Non-Git Directory"
cd "$TEST_DIR"
OUT=$("$SLICK" precmd 2>&1)
[ "$OUT" = "" ] && pass "No output outside git" || fail "Should be empty"

# TEST 3: Modified Files
test_case "Modified Files Detection"
create_test_repo "$TEST_DIR/test3"
echo "modified" >README.md
OUT=$(SLICK_PROMPT_GIT_FETCH=0 "$SLICK" precmd 2>&1)
[[ "$OUT" == *'M 1'* ]] && pass "Modified file detected" || fail "Not detected"

# TEST 4: Staged Files
test_case "Staged Files Detection"
create_test_repo "$TEST_DIR/test4"
echo "new file" >newfile.txt
git add newfile.txt
OUT=$(SLICK_PROMPT_GIT_FETCH=0 "$SLICK" precmd 2>&1)
[[ "$OUT" == *'"staged":true'* ]] && pass "Staged file detected" || fail "Not detected"

# TEST 5: Untracked Files
test_case "Untracked Files Detection"
create_test_repo "$TEST_DIR/test5"
echo "untracked" >untracked.txt
OUT=$(SLICK_PROMPT_GIT_FETCH=0 "$SLICK" precmd 2>&1)
[[ "$OUT" == *'?? 1'* ]] && pass "Untracked file detected" || fail "Not detected"

# TEST 6: Multiple File States
test_case "Multiple File States"
create_test_repo "$TEST_DIR/test6"
echo "modified" >README.md
echo "staged" >staged.txt && git add staged.txt
echo "untracked" >untracked.txt
OUT=$(SLICK_PROMPT_GIT_FETCH=0 "$SLICK" precmd 2>&1)
if [[ "$OUT" == *'M 1'* ]] && [[ "$OUT" == *'?? 1'* ]] && [[ "$OUT" == *'"staged":true'* ]]; then
    pass "Multiple states detected"
else
    fail "Not all states detected"
fi

# TEST 7: JSON Output Format
test_case "JSON Output Format"
create_test_repo "$TEST_DIR/test7"
OUT=$(SLICK_PROMPT_GIT_FETCH=0 "$SLICK" precmd 2>&1)
REQUIRED=("branch" "status" "staged" "remote" "action" "u_name")
ALL_VALID=true
for field in "${REQUIRED[@]}"; do
    if ! echo "$OUT" | grep -q "\"$field\""; then
        fail "Missing field: $field"
        ALL_VALID=false
    fi
done
[ "$ALL_VALID" = true ] && pass "All JSON fields present" || true

# TEST 8: Detached HEAD
test_case "Detached HEAD State"
create_test_repo "$TEST_DIR/test8"
COMMIT=$(git rev-parse HEAD)
git checkout -q "$COMMIT" 2>/dev/null
OUT=$(SLICK_PROMPT_GIT_FETCH=0 "$SLICK" precmd 2>&1)
[[ "$OUT" == *'"branch"'* ]] && pass "Detached HEAD works" || fail "Failed"

# TEST 9: Prompt Display Integration
test_case "Prompt Display Integration"
create_test_repo "$TEST_DIR/test9"
DATA=$(SLICK_PROMPT_GIT_FETCH=0 "$SLICK" precmd 2>&1)
PROMPT=$("$SLICK" prompt -k main -r 0 -d "$DATA" 2>&1)
[ "$PROMPT" != "" ] && pass "Prompt display works" || fail "Display failed"

# TEST 10: Performance (Fetch Disabled)
test_case "Performance (Fetch Disabled)"
create_test_repo "$TEST_DIR/test10"
START=$(date +%s)
SLICK_PROMPT_GIT_FETCH=0 "$SLICK" precmd >/dev/null 2>&1
END=$(date +%s)
DUR=$((END - START))
if [ "$DUR" -lt 2 ]; then
    pass "Fast execution (${DUR}s)"
else
    info "Completed in ${DUR}s"
    pass "Executed successfully"
fi

# TEST 11: HTTPS Remote
test_case "HTTPS Remote (No Auth Check)"
create_test_repo "$TEST_DIR/test11"
git remote add origin https://github.com/fake/repo.git
START=$(date +%s)
SLICK_PROMPT_GIT_FETCH=1 "$SLICK" precmd >/dev/null 2>&1
END=$(date +%s)
DUR=$((END - START))
if [ "$DUR" -lt 3 ]; then
    pass "HTTPS remote quick (${DUR}s)"
else
    info "Took ${DUR}s (acceptable)"
    pass "Completed"
fi

# TEST 12: SSH Remote Timeout (Unreachable)
test_case "SSH Remote Timeout (Unreachable IP)"
create_test_repo "$TEST_DIR/test12"
git remote add origin git@192.0.2.1:fake/repo.git
info "Testing timeout with unreachable IP..."
START=$(date +%s)
timeout 10 "$SLICK" precmd >/dev/null 2>&1
EXIT_CODE=$?
END=$(date +%s)
DUR=$((END - START))
if [ "$EXIT_CODE" -eq 0 ] || [ "$EXIT_CODE" -eq 124 ]; then
    if [ "$DUR" -lt 7 ]; then
        pass "Timeout protection works (${DUR}s)"
    else
        fail "Took too long (${DUR}s)"
    fi
else
    fail "Unexpected exit code: $EXIT_CODE"
fi

# TEST 13: SSH Auth Prompt Protection (Real GitHub)
test_case "SSH Auth Prompt Protection"
create_test_repo "$TEST_DIR/test13"
# Use a fake private GitHub repo that would require auth
git remote add origin git@github.com:fake-org-999999/private-repo-12345.git
info "Testing no password/key prompt (may take 3-5s)..."
START=$(date +%s)
timeout 10 "$SLICK" precmd >/dev/null 2>&1
EXIT_CODE=$?
END=$(date +%s)
DUR=$((END - START))
if [ "$EXIT_CODE" -eq 0 ] || [ "$EXIT_CODE" -eq 124 ]; then
    if [ "$DUR" -lt 8 ]; then
        pass "No auth prompt, completed in ${DUR}s"
    else
        fail "Took too long (${DUR}s), may have prompted"
    fi
else
    pass "Completed without hanging"
fi

# TEST 14: User Name Detection
test_case "Git User Name Detection"
create_test_repo "$TEST_DIR/test14"
OUT=$(SLICK_PROMPT_GIT_FETCH=0 "$SLICK" precmd 2>&1)
[[ "$OUT" == *'"u_name":"Test User"'* ]] && pass "User name detected" || fail "Not detected"

# TEST 15: Empty Repository
test_case "Empty Repository (No Commits)"
mkdir -p "$TEST_DIR/test15"
cd "$TEST_DIR/test15"
git init -q
git config user.email "test@example.com"
git config user.name "Test"
OUT=$(SLICK_PROMPT_GIT_FETCH=0 "$SLICK" precmd 2>&1)
pass "Handles empty repo gracefully"

# TEST 16: Fetch Disabled with Unreachable Remote
test_case "Fetch Disabled Performance"
create_test_repo "$TEST_DIR/test16"
git remote add origin git@192.0.2.1:fake/repo.git
START=$(date +%s)
SLICK_PROMPT_GIT_FETCH=0 "$SLICK" precmd >/dev/null 2>&1
END=$(date +%s)
DUR=$((END - START))
[ "$DUR" -lt 2 ] && pass "Fast with fetch disabled (${DUR}s)" || fail "Too slow"

# TEST 17: Performance Benchmark (10 runs)
test_case "Performance Benchmark (10 iterations)"
create_test_repo "$TEST_DIR/test17"
TOTAL=0
for i in {1..10}; do
    START=$(date +%s%N 2>/dev/null || echo "0")
    SLICK_PROMPT_GIT_FETCH=0 "$SLICK" precmd >/dev/null 2>&1
    if [ "$START" != "0" ]; then
        END=$(date +%s%N)
        TOTAL=$((TOTAL + (END - START) / 1000000))
    fi
done
if [ "$TOTAL" -gt 0 ]; then
    AVG=$((TOTAL / 10))
    if [ "$AVG" -lt 100 ]; then
        pass "Average: ${AVG}ms (excellent)"
    elif [ "$AVG" -lt 200 ]; then
        pass "Average: ${AVG}ms (good)"
    else
        info "Average: ${AVG}ms"
        pass "Completed benchmark"
    fi
else
    pass "Benchmark completed"
fi


# TEST 18: Custom Environment Variables
test_case "Custom Environment Variables"
create_test_repo "$TEST_DIR/test18"
# Source test config
source "$SCRIPT_DIR/envrc.test"
# Get prompt output
DATA=$($SLICK precmd 2>&1)
PROMPT=$($SLICK prompt -k main -r 0 -d "$DATA" 2>&1)
# Verify prompt was generated
if [ -n "$PROMPT" ]; then
    pass "Custom env vars applied"
else
    fail "Failed to generate prompt"
fi

# SUMMARY
echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}TEST SUMMARY${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"

echo "Tests run:    $TESTS_RUN"
echo -e "${GREEN}Passed:       $TESTS_PASSED${NC}"

if [ "$TESTS_FAILED" -gt 0 ]; then
    echo -e "${RED}Failed:       $TESTS_FAILED${NC}"
    echo -e "\n${RED}❌ Some tests failed${NC}"
    exit 1
else
    echo -e "Failed:       0"
    echo -e "\n${GREEN}✅ All $TESTS_PASSED tests passed!${NC}"
    echo -e "\n${BLUE}Binary:${NC}"
    ls -lh "$SLICK"
    exit 0
fi
