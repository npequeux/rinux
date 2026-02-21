# Release Checklist for Rinux vX.Y.Z

This checklist should be completed before creating each release.

## Pre-Release (Development Phase)

### Code Quality
- [ ] All CI checks passing on main branch
- [ ] No compiler warnings (`cargo clippy --all`)
- [ ] Code formatted correctly (`cargo fmt --all`)
- [ ] CodeQL security scan passes with no vulnerabilities
- [ ] All unit tests pass (`make test`)
- [ ] Manual testing in QEMU (`make run`)

### Documentation
- [ ] README.md updated with current features
- [ ] FEATURES.md reflects all implemented functionality
- [ ] CHANGELOG.md includes all changes since last release
- [ ] Release notes document created (for major/minor releases)
- [ ] API documentation builds without errors (`cargo doc`)
- [ ] All new public APIs have doc comments

### Version Management
- [ ] Version number updated in `Cargo.toml`
- [ ] CHANGELOG.md has section for new version with date
- [ ] Version follows semantic versioning correctly
  - MAJOR: Breaking changes
  - MINOR: New features, backwards compatible
  - PATCH: Bug fixes only

### Code Review
- [ ] All new code has been reviewed
- [ ] Security-sensitive changes reviewed thoroughly
- [ ] Breaking changes documented and justified

## Release Preparation

### Final Checks
- [ ] Create a fresh clone and verify build works
  ```bash
  git clone https://github.com/npequeux/rinux.git /tmp/rinux-test
  cd /tmp/rinux-test
  make build
  make test
  ```
- [ ] Verify kernel boots in QEMU
- [ ] Check for any TODO comments in critical sections
- [ ] Review git log since last release
  ```bash
  git log v0.1.0..HEAD --oneline
  ```

### Documentation Review
- [ ] Proofread CHANGELOG.md entry
- [ ] Verify all links in documentation work
- [ ] Check that feature list matches implementation
- [ ] Ensure migration guide exists (if breaking changes)

### Security Review
- [ ] Review all uses of `unsafe`
- [ ] Verify input validation on external data
- [ ] Check for potential panic conditions
- [ ] Review error handling paths

## Creating the Release

### Tag Creation
- [ ] Ensure main branch is up to date
  ```bash
  git checkout main
  git pull origin main
  ```
- [ ] Verify no uncommitted changes
  ```bash
  git status
  ```
- [ ] Run release helper script OR manually create tag
  ```bash
  ./scripts/release.sh release X.Y.Z
  # OR
  git tag -a vX.Y.Z -m "Release version X.Y.Z"
  git push origin vX.Y.Z
  ```

### Monitor Release Build
- [ ] Watch GitHub Actions release workflow
  - URL: https://github.com/npequeux/rinux/actions
- [ ] Verify all build steps complete successfully
- [ ] Check that artifacts are uploaded

### Verify Release
- [ ] Confirm release appears on GitHub
  - URL: https://github.com/npequeux/rinux/releases
- [ ] Download and verify release artifacts
  ```bash
  wget https://github.com/npequeux/rinux/releases/download/vX.Y.Z/rinux-vX.Y.Z-x86_64.tar.gz
  wget https://github.com/npequeux/rinux/releases/download/vX.Y.Z/checksums.txt
  sha256sum -c checksums.txt
  ```
- [ ] Extract and inspect archive contents
  ```bash
  tar -xzf rinux-vX.Y.Z-x86_64.tar.gz
  ls -la
  ```

### Update Release Page
- [ ] Edit release on GitHub if needed
- [ ] Add highlights and key changes to description
- [ ] Link to detailed release notes
- [ ] Add migration instructions if applicable
- [ ] Mark as pre-release if alpha/beta/rc

## Post-Release

### Communication
- [ ] Announce release on project channels
- [ ] Update project website (if applicable)
- [ ] Post to relevant communities
- [ ] Update social media (optional)

### Repository Updates
- [ ] Update roadmap to reflect completed items
  - File: `docs/ROADMAP.md`
- [ ] Archive release notes (if not already in docs/)
- [ ] Close completed issues linked to this release
- [ ] Close milestone for this version

### Planning Next Release
- [ ] Create milestone for next version
- [ ] Plan features for next release
- [ ] Update ROADMAP.md with next version goals
- [ ] Consider incrementing version to X.Y.Z-dev in Cargo.toml

## Rollback Plan (If Issues Found)

### Immediate Actions
- [ ] Do NOT delete the release (it should remain visible)
- [ ] Add prominent warning to release notes
- [ ] Create GitHub issue documenting the problem
- [ ] Plan hotfix release if critical bug found

### Communication
- [ ] Notify users via all channels
- [ ] Explain the issue clearly
- [ ] Provide workaround if available
- [ ] Give timeline for hotfix

## Sign-Off

Release prepared by: _______________
Date: _______________

Final review by: _______________
Date: _______________

Release created: _______________
Date: _______________

---

## Notes

**Version**: vX.Y.Z
**Type**: [Major/Minor/Patch]
**Date**: YYYY-MM-DD

### Key Changes
- Change 1
- Change 2
- Change 3

### Known Issues
- Issue 1
- Issue 2

### Special Instructions
- Any special notes for this release

---

**Template Version**: 1.0
**Last Updated**: February 21, 2026
