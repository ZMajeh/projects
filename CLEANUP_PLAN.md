# Repo Cleanup Plan

> Deferred on 2026-10-18 — revisit when ready. Goal: reduce repo bloat (370 MiB pack,
> 45,051 tracked files) and stop global CI triggers, without changing the monorepo layout.

## Current Problems (measured 2026-10-18)

- Git root `/home/majeh/git/projects` holds 16 projects sharing one history.
- No root `.gitignore`; per-project ignores are incomplete (only `Majeh's Email Browserr/.gitignore` exists, ignoring `firebase.ts`).
- Build artifacts tracked in git:
  - `Majeh's Email Browserr/target/` — 1,874 files, incl. two 50 MB `.wasm` debug binaries and 27–35 MB `.so`/`.rlib` files
  - `Majeh's Email Browserr/dist/` — 14,295 files (committed build output)
  - `lodgeManagement/node_modules/` — incl. 27 MB compiled `re2.node`
  - 183 binary files total (`.wasm/.so/.dll/.exe/.o/.a`)
- `.github/workflows/deploy.yml` triggers on **every** push to `main`, rebuilding and deploying Majeh's Email Browserr to Firebase Hosting regardless of which project changed.

## Option A — Cleanup Steps (recommended)

### 1. Root `.gitignore`

Create `/home/majeh/git/projects/.gitignore`:

```gitignore
# Rust
target/

# Frontend build output
dist/
node_modules/

# C/C++ / Qt / misc build artifacts
build/
cmake-build-*/
*.o
*.so
*.a
*.dll
*.exe
*.rlib

# Python
__pycache__/
*.pyc
.venv/

# OS / editor
.DS_Store
*.swp
```

(Review per-project needs — some projects may track `dist/` intentionally; adjust before committing.)

### 2. Untrack artifacts (files stay on disk, removed from git index only)

```bash
cd /home/majeh/git/projects
git rm -r --cached "Majeh's Email Browserr/target"
git rm -r --cached "Majeh's Email Browserr/dist"
git rm -r --cached lodgeManagement/node_modules
# find and remove any other tracked binaries/artifacts:
git ls-files | grep -E '(^|/)(target|dist|node_modules|build)/' | xargs -d '\n' git rm --cached
```

### 3. Path-filter CI triggers

Edit `.github/workflows/deploy.yml`:

```yaml
on:
  push:
    branches:
      - main
    paths:
      - "Majeh's Email Browserr/**"
      - ".github/workflows/deploy.yml"
```

### 4. Commit & verify

```bash
git commit -m "chore: add root .gitignore, untrack build artifacts, path-filter deploy CI"
git count-objects -vH   # pack size won't shrink yet — history still holds old blobs
```

### 5. (Optional) Shrink history with `git filter-repo`

```bash
# WARNING: rewrites ALL commit hashes — coordinate with any clones/CI first
pip install git-filter-repo
git filter-repo --analyze   # inspect blob sizes first
git filter-repo --path "Majeh's Email Browserr/target" --invert-paths
git filter-repo --path "Majeh's Email Browserr/dist" --invert-paths
git filter-repo --path lodgeManagement/node_modules --invert-paths
# then force-push and re-clone on all machines
```

## Decision Log

- **2026-10-18:** Monorepo kept (Option B — one repo per project — rejected for now: too much
  solo-dev overhead, breaks shared history, doesn't fix bloat without per-repo cleanup anyway).
  Cleanup deferred; this file is the reference plan for later.
