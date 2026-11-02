# Releasing

A release is a tag push. `.github/workflows/release.yml` does the rest.

## Cut a release

1. Set `version` in `Cargo.toml` to the new version, for example `0.2.0`. Run `gmake build` so `Cargo.lock` follows.
2. In `CHANGELOG.md`, rename `## Unreleased` to `## 0.2.0 (YYYY-MM-DD)`. That section becomes the release notes.
3. Run `gmake check` and `gmake test`. Merge the change to `main` through a pull request.
4. Tag the merge commit on `main` and push the tag:

```sh
git checkout main && git pull --ff-only
git tag v0.2.0
git push origin v0.2.0
```

The tag must equal `v` plus the `Cargo.toml` version. The build fails if they differ.

## What the workflow does

- **build:** one job per target, on GitHub-hosted runners. Each job builds with `--locked` and the default features
  (`model` and `mcp`), checks the binary's linkage, and packages `sloperator-<version>-<target>.tar.gz` with a
  `.sha256` file. The Linux jobs also build `sloperator_<version>_<arch>.deb` with `cargo deb`.

| Target | Runner | Features |
|--------|--------|----------|
| `aarch64-apple-darwin` | `macos-15` | default + `metal` |
| `x86_64-unknown-linux-gnu` | `ubuntu-24.04` | default |
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` | default |

- **release:** creates the GitHub Release for the tag. It attaches every artifact and takes the notes from the
  version's `CHANGELOG.md` section (`packaging/changelog-section.sh`).
- **formula:** renders `Formula/sloperator.rb` from `packaging/sloperator.rb.tmpl` with the release's sha256 values
  (`packaging/render-formula.sh`). It runs `ruby -c` and `brew style`, installs the formula, runs `brew test`, then
  commits `chore(release): formula for vX.Y.Z` and pushes it to `main` with the built-in `GITHUB_TOKEN`.

A pull request that changes the workflow, `Cargo.toml`, `Cargo.lock` or `packaging/` runs build and formula render.
It publishes nothing and pushes nothing.

Users tap this repository by URL. The formula lives in `Formula/` here, so no separate tap repository is needed.

## When a job fails

- **A build job fails.** Nothing is published, because release needs every build. Fix the cause on a branch and
  merge it. Then delete the tag and push it again on the new commit:

```sh
git push origin :refs/tags/v0.2.0
git tag -d v0.2.0
git tag v0.2.0 && git push origin v0.2.0
```

- **release fails.** The binaries are still in the run's artifacts. Fix the cause and choose **Re-run failed jobs** on
  the run. If a half-made release exists, delete it on the releases page first.
- **formula fails, or its push to `main` is refused.** The release is already public. Re-run the failed job. If branch
  protection refused the push, render the formula locally and merge it through a pull request:

```sh
mkdir -p dist Formula
for t in aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
  curl -fsSLo "dist/sloperator-0.2.0-$t.tar.gz.sha256" \
    "https://github.com/bradodarb/sloperator/releases/download/v0.2.0/sloperator-0.2.0-$t.tar.gz.sha256"
done
packaging/render-formula.sh 0.2.0 dist > Formula/sloperator.rb
```

Never move a tag after its release is public. Ship a new patch version instead.
