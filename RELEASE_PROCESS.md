# Rinux Release Process

This document describes the process for creating and publishing releases of Rinux.

## Table of Contents

- [Overview](#overview)
- [Release Workflow](#release-workflow)
- [Version Numbers](#version-numbers)
- [Pre-Release Checklist](#pre-release-checklist)
- [Creating a Release](#creating-a-release)
- [Post-Release Tasks](#post-release-tasks)
- [Hotfix Releases](#hotfix-releases)

---

## Overview

Rinux follows [Semantic Versioning](https://semver.org/) (SemVer) for version numbers and uses Git tags to trigger automated release builds via GitHub Actions.

### Release Types

- **Major Release** (x.0.0): Breaking changes, major new features
- **Minor Release** (0.x.0): New features, backwards compatible
- **Patch Release** (0.0.x): Bug fixes, backwards compatible

---

## Release Workflow

### Automated Release Pipeline

When a version tag is pushed to GitHub, the following happens automatically:

1. **Build**: Kernel is compiled in release mode for x86_64
2. **Package**: Binary and documentation are packaged into a tar.gz archive
3. **Checksum**: SHA256 checksum is generated
4. **Publish**: GitHub Release is created with artifacts attached

### Manual Steps

The following must be done manually before tagging:

1. Update version numbers
2. Update CHANGELOG.md
3. Create/update release notes
4. Run tests and verification
5. Create and push the Git tag

---

## Version Numbers

### Files to Update

Before releasing, update the version number in these files:

1. **Cargo.toml** (main package)
   ```toml
   [package]
   version = "0.2.0"
   ```

2. **CHANGELOG.md** (add new version section)
   ```markdown
   ## [0.2.0] - 2026-02-21
   ```

3. **Release notes** (create if major/minor release)
   - Create `RELEASE_NOTES_vX.Y.Z.md` for major/minor releases
   - Update README.md if features changed significantly

### Version Naming Convention

- Format: `MAJOR.MINOR.PATCH`
- Examples:
  - `0.1.0` - Initial release
  - `0.2.0` - Second minor release
  - `0.2.1` - First patch on 0.2.x
  - `1.0.0` - First stable release

### Pre-release Versions

For testing before official release:

- Alpha: `0.2.0-alpha.1`
- Beta: `0.2.0-beta.1`
- Release Candidate: `0.2.0-rc.1`

---

## Pre-Release Checklist

Complete these steps before creating a release:

### 1. Code Quality ✅

- [ ] All CI checks passing (lint, build, test)
- [ ] No compiler warnings (`cargo clippy` clean)
- [ ] Code formatting is correct (`cargo fmt`)
- [ ] Security scan passes (CodeQL)

### 2. Testing ✅

- [ ] All unit tests pass (`make test`)
- [ ] Manual testing in QEMU (`make run`)
- [ ] Test on real hardware (if applicable)
- [ ] Verify all documented features work

### 3. Documentation ✅

- [ ] README.md is up to date
- [ ] CHANGELOG.md includes all changes
- [ ] FEATURES.md reflects current state
- [ ] API documentation is complete (`cargo doc`)
- [ ] Release notes prepared (for major/minor releases)

### 4. Version Updates ✅

- [ ] Version in Cargo.toml updated
- [ ] CHANGELOG.md has new version section
- [ ] Release notes created (if needed)
- [ ] Date in CHANGELOG is correct

### 5. Build Verification ✅

```bash
# Clean build to verify everything compiles
make clean
make build

# Verify the binary exists and is correct size
ls -lh target/x86_64-unknown-rinux/release/rinux

# Test in QEMU
make run
```

### 6. Final Review ✅

- [ ] Review git diff since last release
- [ ] Verify no sensitive data in commits
- [ ] Check that LICENSE is correct
- [ ] Verify contributor attributions

---

## Creating a Release

### Step 1: Commit Final Changes

```bash
# Make sure all changes are committed
git status

# Commit any remaining changes
git add .
git commit -m "Prepare for v0.2.0 release"

# Push to main/master branch
git push origin main
```

### Step 2: Create a Git Tag

```bash
# Create an annotated tag (recommended)
git tag -a v0.2.0 -m "Release version 0.2.0"

# Or create a lightweight tag
git tag v0.2.0

# Verify the tag was created
git tag -l
```

### Step 3: Push the Tag

```bash
# Push the tag to GitHub (this triggers the release workflow)
git push origin v0.2.0
```

**Important**: Only push the tag when you're ready to release. Once pushed, the CI/CD pipeline will automatically build and publish the release.

### Step 4: Monitor the Release Build

1. Go to GitHub Actions: https://github.com/npequeux/rinux/actions
2. Look for the "Release" workflow run
3. Monitor the build progress
4. Verify all steps complete successfully

### Step 5: Verify the GitHub Release

1. Go to Releases: https://github.com/npequeux/rinux/releases
2. Find the new release (should be at the top)
3. Verify the release includes:
   - Release notes (auto-generated from tag message or workflow)
   - Kernel binary archive (`rinux-vX.Y.Z-x86_64.tar.gz`)
   - Checksums file (`checksums.txt`)
4. Download and verify the archive:

```bash
# Download the release
wget https://github.com/npequeux/rinux/releases/download/v0.2.0/rinux-v0.2.0-x86_64.tar.gz
wget https://github.com/npequeux/rinux/releases/download/v0.2.0/checksums.txt

# Verify checksum
sha256sum -c checksums.txt

# Extract and inspect
tar -xzf rinux-v0.2.0-x86_64.tar.gz
ls -la
```

### Step 6: Update the Release Notes (Optional)

If the auto-generated release notes need enhancement:

1. Go to the release page on GitHub
2. Click "Edit release"
3. Update the description with:
   - Highlights of major changes
   - Breaking changes (if any)
   - Known issues
   - Upgrade instructions
   - Links to detailed release notes
4. Save the changes

Example release description:
```markdown
# Rinux v0.2.0 - Memory Master

## Highlights
- Advanced memory management with slab allocator
- Fully functional TmpFS filesystem
- Enhanced device drivers (serial, keyboard, graphics)
- GPU support for Intel, AMD, and NVIDIA

## What's Changed
See [CHANGELOG.md](https://github.com/npequeux/rinux/blob/v0.2.0/CHANGELOG.md) for detailed changes.

## Known Issues
- Process fork/exec not yet functional
- Block device DMA operations not implemented
- No network stack

## Documentation
- [Features](https://github.com/npequeux/rinux/blob/v0.2.0/FEATURES.md)
- [Release Notes](https://github.com/npequeux/rinux/blob/v0.2.0/RELEASE_NOTES_v0.2.0.md)
- [Roadmap](https://github.com/npequeux/rinux/blob/v0.2.0/docs/ROADMAP.md)

**Full Changelog**: https://github.com/npequeux/rinux/compare/v0.1.0...v0.2.0
```

---

## Post-Release Tasks

After a successful release:

### 1. Announce the Release

- [ ] Post announcement on project communication channels
- [ ] Update project website (if applicable)
- [ ] Share on social media (optional)

### 2. Update Development Branch

```bash
# Start work on next version
git checkout -b develop

# Update Cargo.toml with next version
# For example, if you just released 0.2.0, set to 0.3.0-dev
```

### 3. Create Milestone for Next Release

1. Go to GitHub Issues
2. Create a new milestone (e.g., "v0.3.0")
3. Add planned features and bugs to the milestone

### 4. Update Documentation

- [ ] Update ROADMAP.md to reflect completed items
- [ ] Archive old release notes (move to docs/releases/)
- [ ] Update main README if needed

---

## Hotfix Releases

For critical bug fixes that need immediate release:

### Process

1. **Create hotfix branch** from the release tag:
   ```bash
   git checkout -b hotfix/v0.2.1 v0.2.0
   ```

2. **Fix the bug**:
   ```bash
   # Make necessary fixes
   git commit -am "Fix critical bug in memory allocator"
   ```

3. **Update version**:
   - Update Cargo.toml to patch version (0.2.1)
   - Add CHANGELOG entry

4. **Test thoroughly**:
   ```bash
   make test
   make run
   ```

5. **Merge to main**:
   ```bash
   git checkout main
   git merge hotfix/v0.2.1
   git push origin main
   ```

6. **Tag and release**:
   ```bash
   git tag -a v0.2.1 -m "Hotfix: Fix memory allocator bug"
   git push origin v0.2.1
   ```

7. **Merge back to develop**:
   ```bash
   git checkout develop
   git merge hotfix/v0.2.1
   git push origin develop
   ```

---

## Troubleshooting

### Release Build Fails

If the automated release build fails:

1. Check the GitHub Actions logs
2. Fix the issue in the codebase
3. Delete the tag locally and remotely:
   ```bash
   git tag -d v0.2.0
   git push origin :refs/tags/v0.2.0
   ```
4. Fix the issue and commit
5. Create the tag again

### Wrong Version Released

If you released the wrong version:

1. **Don't delete the GitHub release** (releases should be permanent)
2. Create a new patch release with fixes
3. Add a note to the incorrect release explaining the issue

### Checksums Don't Match

If checksums don't match after download:

1. Verify download completed successfully
2. Re-download the file
3. If still failing, report as an issue
4. May need to create a hotfix release

---

## Release Automation Details

### GitHub Actions Workflow

Location: `.github/workflows/release.yml`

**Triggers**: Push of tags matching `v*.*.*`

**Steps**:
1. Checkout code
2. Install Rust nightly with required components
3. Build kernel in release mode
4. Create release archive with binary and documentation
5. Generate SHA256 checksums
6. Create GitHub Release with artifacts

**Artifacts**:
- `rinux-vX.Y.Z-x86_64.tar.gz` - Kernel binary and docs
- `checksums.txt` - SHA256 checksums

**Release Type Detection**:
- Pre-release if tag contains: `alpha`, `beta`, or `rc`
- Production release otherwise

---

## Best Practices

1. **Always test before tagging**: Once a tag is pushed, the release is public
2. **Use annotated tags**: They contain more metadata than lightweight tags
3. **Follow SemVer strictly**: Users rely on version numbers for compatibility
4. **Write good release notes**: Help users understand what changed
5. **Keep CHANGELOG current**: Update it with every significant change
6. **Never delete releases**: They should be permanent (except in extreme cases)
7. **Plan releases**: Have a roadmap and stick to it
8. **Communicate**: Let users know about upcoming releases

---

## Release Schedule

Rinux follows a time-based release schedule:

- **Minor releases**: Every 2-3 months
- **Patch releases**: As needed for critical bugs
- **Major releases**: When breaking changes accumulate

### Planned Release Dates

- v0.2.0: February 2026 ✅
- v0.3.0: May 2026 (planned)
- v0.4.0: August 2026 (planned)
- v0.5.0: November 2026 (planned)
- v1.0.0: 2027 (tentative)

---

## Support Policy

- **Latest version**: Full support with active development
- **Previous minor version**: Security fixes only for 6 months
- **Older versions**: No support (upgrade recommended)

Example:
- If v0.3.0 is released, v0.2.x receives security fixes until v0.4.0
- v0.1.x is no longer supported once v0.2.0 is released

---

## Questions?

If you have questions about the release process:
- Check existing GitHub Issues
- Open a new issue with the "question" label
- Contact the maintainers

---

**Maintainers**: @npequeux
**Last Updated**: February 21, 2026
