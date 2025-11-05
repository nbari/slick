# Testing Slick Prompt

## Quick Start

```bash
# Run all tests
just test

# Or run directly
./test.sh
```

## Test Coverage

### Integration Tests (17 tests)

**Basic Functionality:**
1. Basic git repository detection
2. Non-git directory handling
3. Modified files detection
4. Staged files detection  
5. Untracked files detection
6. Multiple file states
7. JSON output format validation

**Edge Cases:**
8. Detached HEAD state
9. Prompt display integration
10. Empty repository handling

**Performance:**
11. Performance (fetch disabled)
16. Fetch disabled with unreachable remote
17. Performance benchmark (10 iterations)

**Network & Auth Protection:** ⚡
11. HTTPS remote handling
12. **SSH remote timeout** (unreachable IP)
13. **SSH auth prompt protection** (prevents password/key prompts) 🔒
14. Git user name detection

### Unit Tests (18 tests)

```bash
cargo test
```

- 3 unit tests (src/lib.rs)
- 7 environment tests  
- 8 git integration tests

**Total: 35 tests** (17 integration + 18 unit)

## Auth/Credential Protection

Test #13 verifies that slick **never prompts for credentials**:

```bash
# What it tests:
git remote add origin git@github.com:fake-org/private-repo.git
slick precmd  # Should NOT prompt for password/SSH key

# Protection mechanisms tested:
- GIT_TERMINAL_PROMPT=0
- GIT_ASKPASS=echo
- SSH_ASKPASS=echo  
- SSH BatchMode=yes
- Timeout protection
```

**Result:** ✅ No hanging, no password prompts, completes quickly

## Justfile Commands

```bash
just test        # Clippy + cargo tests + integration tests
just check       # All tests + format check
just clippy      # Run clippy with strict warnings
just build       # Build release binary
just fmt         # Format code
just clean       # Clean build artifacts
just integration # Run integration tests only
just version     # Show version
```

## Manual Testing

### Test Auth Prompt Prevention

```bash
# Create repo with SSH remote that requires auth
mkdir /tmp/test-auth && cd /tmp/test-auth
git init
git config user.email "test@test.com"
git config user.name "Test"
echo "test" > file.txt
git add . && git commit -m "init"

# Add private repo remote
git remote add origin git@github.com:some-org/private-repo.git

# Should complete in < 5 seconds without prompting
time slick precmd
```

### Test Timeout Protection

```bash
# Add unreachable remote
git remote add origin git@192.0.2.1:fake/repo.git

# Should complete in < 6 seconds
time slick precmd
```

## What Gets Tested

| Scenario | Test # | What It Verifies |
|----------|--------|------------------|
| SSH unreachable | 12 | Timeout works (no hang) |
| SSH auth required | 13 | No password prompt, no hang |
| HTTPS remote | 11 | Quick completion |
| No remote | 1-10 | Normal git operations |
| Fetch disabled | 16 | Skips network entirely |

## CI/CD

```bash
just check
```

Runs:
- ✅ Cargo clippy (strict)
- ✅ Cargo tests (18 tests)
- ✅ Format check
- ✅ Integration tests (17 tests)

## Performance Expectations

| Test | Expected Time |
|------|---------------|
| Local operations (fetch disabled) | < 100ms |
| HTTPS remote | < 3s |
| SSH unreachable | < 6s |
| SSH auth required | < 8s |
| Benchmark average | < 100ms |

## Summary

**Simple workflow:**
```bash
just test    # Everything
```

**Total coverage:** 35 tests
- 18 Cargo tests ✅
- 17 Integration tests ✅
  - Including **SSH auth prompt protection** 🔒
  - Including **SSH timeout protection** ⚡
- All passing ✅

**Key Features Tested:**
- ✅ No password prompts
- ✅ No SSH key prompts  
- ✅ Timeout protection
- ✅ Fast execution
- ✅ Complete git status detection
