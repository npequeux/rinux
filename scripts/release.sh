#!/bin/bash

# Rinux Release Helper Script
# This script automates the release process for Rinux

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored messages
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."
    
    local missing_tools=()
    
    if ! command_exists git; then
        missing_tools+=("git")
    fi
    
    if ! command_exists cargo; then
        missing_tools+=("cargo")
    fi
    
    if ! command_exists rustc; then
        missing_tools+=("rustc")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
    
    print_success "All prerequisites satisfied"
}

# Function to get current version from Cargo.toml
get_current_version() {
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Function to check if on main branch
check_branch() {
    local branch=$(git rev-parse --abbrev-ref HEAD)
    if [ "$branch" != "main" ] && [ "$branch" != "master" ]; then
        print_warning "You are on branch '$branch', not 'main' or 'master'"
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# Function to check for uncommitted changes
check_uncommitted() {
    if ! git diff-index --quiet HEAD --; then
        print_error "You have uncommitted changes. Please commit or stash them first."
        git status --short
        exit 1
    fi
}

# Function to run pre-release checks
pre_release_checks() {
    print_info "Running pre-release checks..."
    
    # Check formatting
    print_info "Checking code formatting..."
    if ! cargo +nightly fmt --all -- --check; then
        print_error "Code formatting check failed. Run 'cargo fmt' to fix."
        exit 1
    fi
    print_success "Code formatting OK"
    
    # Check clippy
    print_info "Running clippy..."
    if ! cargo +nightly clippy --lib --bins --all-features -- -D warnings; then
        print_error "Clippy found issues. Please fix them before releasing."
        exit 1
    fi
    print_success "Clippy checks passed"
    
    # Build in release mode
    print_info "Building in release mode..."
    if ! cargo +nightly build --release; then
        print_error "Release build failed."
        exit 1
    fi
    print_success "Release build successful"
    
    print_success "All pre-release checks passed!"
}

# Function to create a release
create_release() {
    local version="$1"
    
    if [ -z "$version" ]; then
        print_error "Version is required"
        echo "Usage: $0 release <version>"
        echo "Example: $0 release 0.2.0"
        exit 1
    fi
    
    # Add 'v' prefix if not present
    if [[ ! "$version" =~ ^v ]]; then
        version="v$version"
    fi
    
    print_info "Creating release $version"
    
    # Check prerequisites
    check_prerequisites
    check_branch
    check_uncommitted
    
    # Get current version
    local current_version=$(get_current_version)
    print_info "Current version in Cargo.toml: $current_version"
    
    # Verify version matches
    if [ "$version" != "v$current_version" ]; then
        print_warning "Version mismatch!"
        print_warning "  Tag version: $version"
        print_warning "  Cargo.toml:  v$current_version"
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Run pre-release checks
    pre_release_checks
    
    # Confirm release
    echo ""
    print_warning "About to create release $version"
    print_info "This will:"
    print_info "  1. Create an annotated git tag"
    print_info "  2. Push the tag to GitHub"
    print_info "  3. Trigger the automated release workflow"
    echo ""
    read -p "Proceed with release? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Release cancelled."
        exit 0
    fi
    
    # Create the tag
    print_info "Creating git tag $version..."
    if ! git tag -a "$version" -m "Release $version"; then
        print_error "Failed to create tag"
        exit 1
    fi
    print_success "Tag created: $version"
    
    # Push the tag
    print_info "Pushing tag to GitHub..."
    if ! git push origin "$version"; then
        print_error "Failed to push tag"
        print_warning "You can manually push the tag with: git push origin $version"
        print_warning "Or delete it with: git tag -d $version"
        exit 1
    fi
    print_success "Tag pushed to GitHub"
    
    echo ""
    print_success "Release $version created successfully!"
    print_info "GitHub Actions will now build and publish the release."
    print_info "Check the progress at: https://github.com/npequeux/rinux/actions"
    print_info "Release will be available at: https://github.com/npequeux/rinux/releases/tag/$version"
}

# Function to prepare for a release (update version)
prepare_release() {
    local new_version="$1"
    
    if [ -z "$new_version" ]; then
        print_error "Version is required"
        echo "Usage: $0 prepare <version>"
        echo "Example: $0 prepare 0.3.0"
        exit 1
    fi
    
    # Remove 'v' prefix if present
    new_version="${new_version#v}"
    
    print_info "Preparing for release v$new_version"
    
    # Check prerequisites
    check_prerequisites
    check_branch
    
    # Get current version
    local current_version=$(get_current_version)
    print_info "Current version: $current_version"
    print_info "New version: $new_version"
    
    # Confirm version update
    read -p "Update version to $new_version? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Cancelled."
        exit 0
    fi
    
    # Update Cargo.toml
    print_info "Updating Cargo.toml..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    fi
    print_success "Cargo.toml updated"
    
    # Get today's date
    local date=$(date +%Y-%m-%d)
    
    # Add section to CHANGELOG.md if not already present
    if ! grep -q "## \[$new_version\]" CHANGELOG.md; then
        print_info "Adding section to CHANGELOG.md..."
        # Insert after the first line (header)
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "6i\\
\\
## [$new_version] - $date\\
\\
### Added\\
\\
### Changed\\
\\
### Fixed\\
\\
### Deprecated\\
\\
### Removed\\
\\
### Security\\
" CHANGELOG.md
        else
            sed -i "6i\\\\n## [$new_version] - $date\\n\\n### Added\\n\\n### Changed\\n\\n### Fixed\\n\\n### Deprecated\\n\\n### Removed\\n\\n### Security\\n" CHANGELOG.md
        fi
        print_success "CHANGELOG.md updated"
    else
        print_info "CHANGELOG.md already has section for $new_version"
    fi
    
    # Show what changed
    echo ""
    print_info "Changes made:"
    git diff Cargo.toml CHANGELOG.md
    
    echo ""
    print_success "Version prepared for v$new_version"
    print_info "Next steps:"
    print_info "  1. Update CHANGELOG.md with actual changes"
    print_info "  2. Review and commit changes: git commit -am 'Prepare for v$new_version release'"
    print_info "  3. Run: $0 release $new_version"
}

# Function to show status
show_status() {
    print_info "Rinux Release Status"
    echo ""
    
    # Get current version
    local version=$(get_current_version)
    print_info "Current version: $version"
    
    # Get latest tag
    local latest_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "No tags")
    print_info "Latest tag: $latest_tag"
    
    # Check branch
    local branch=$(git rev-parse --abbrev-ref HEAD)
    print_info "Current branch: $branch"
    
    # Check for uncommitted changes
    if git diff-index --quiet HEAD --; then
        print_success "Working directory clean"
    else
        print_warning "Uncommitted changes present"
    fi
    
    echo ""
    
    # Show recent tags
    print_info "Recent tags:"
    git tag -l --sort=-v:refname | head -5 || print_warning "No tags found"
}

# Main script
case "$1" in
    release)
        create_release "$2"
        ;;
    prepare)
        prepare_release "$2"
        ;;
    status)
        show_status
        ;;
    *)
        echo "Rinux Release Helper"
        echo ""
        echo "Usage: $0 <command> [arguments]"
        echo ""
        echo "Commands:"
        echo "  status              Show current release status"
        echo "  prepare <version>   Prepare for a new release (update version files)"
        echo "  release <version>   Create and publish a release"
        echo ""
        echo "Examples:"
        echo "  $0 status"
        echo "  $0 prepare 0.3.0"
        echo "  $0 release 0.2.0"
        echo ""
        echo "For more information, see RELEASE_PROCESS.md"
        exit 1
        ;;
esac
