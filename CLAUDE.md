# Claude Code Rules

**Primary Directive**: Claude is responsible for applying, maintaining, and documenting all rules in this file. These rules are non-negotiable and must be followed at all times.

## Code Quality

### No Lazy Solutions
- Always choose the clean, correct solution over the quick hack
- If there's an easy way and a good way, choose the good way
- Prefer maintainable code over clever shortcuts
- Technical debt is not acceptable; fix it properly the first time

### Testing Requirements
- **Minimum coverage: 80%** - CI must fail if coverage drops below this threshold
- **Target coverage: 100%** - Strive for complete test coverage
- Every new feature must include tests
- Every bug fix must include a regression test
- Tests must be meaningful, not just coverage padding

## Development Standards

### Rust Specific
- All code must pass `cargo fmt --check`
- All code must pass `cargo clippy -- -D warnings`
- No `#[allow(...)]` without documented justification
- Prefer safe Rust; `unsafe` requires detailed safety comments

### Documentation
- Public APIs must have doc comments
- Complex logic must have explanatory comments
- Architecture decisions must be documented in `docs/`

### Git Practices
- Commits must be atomic and well-described
- No commits that break the build
- Pre-commit hooks must pass before pushing

## CI/CD Requirements

- All CI checks must pass before merging
- Coverage reports must be generated and tracked
- WASM builds must succeed and stay under size budget

---

*Last updated: 2025-01-20*
*Maintainer: Claude Code*
