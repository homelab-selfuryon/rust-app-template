# rust-app-template

A GitHub template repository for a Rust app on the
[homelab-selfuryon](https://github.com/homelab-selfuryon) platform. It is deliberately thin: everything
that must evolve comes as a pinned, Renovate-bumped dependency, not as code you own:

| From                                                                        | What                                                                                                                                                                                     |
| --------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`platform-kit`](https://github.com/homelab-selfuryon/platform-kit)         | the CUE schema of `deploy/deploy.cue` and its render to Kubernetes objects; the devenv module: toolchain, tools, treefmt, git hooks, local Postgres, and `devenv test` (the CI contract) |
| [`github-workflows`](https://github.com/homelab-selfuryon/github-workflows) | CI, preview, release, promote, register: the workflows here only call them                                                                                                               |
| [`.github`](https://github.com/homelab-selfuryon/.github)                   | the Renovate preset                                                                                                                                                                      |

After "Use this template" the Cargo files and the code are yours; there is no template sync.

## What is in it

- **axum 0.8 + tokio**, JSON logs (`tracing`), `/healthz` (liveness) and `/readyz` (readiness, fails while
  shutting down), graceful shutdown on SIGTERM (not ready, drain for `SHUTDOWN_DELAY_SECS`, stop).
- **Configuration** (`src/config.rs`): non-secret settings from environment variables, set in `env:` of
  `deploy.cue`. Secrets are **files**, never variables: `src/secrets.rs` reads them from
  `/var/run/secrets/app/<key>` (OpenBao through the CSI driver) and `/var/run/secrets/db/` and wraps them
  in `secrecy::SecretString`.
- **Service-to-service authentication** (`src/auth/`): verifies a caller's projected ServiceAccount
  token. The signing keys come from the kube-apiserver (`/openid/v1/jwks`, fetched with the pod's own
  token and the cluster CA, cached, re-fetched on rotation, rate limited). A token must carry this
  app's name as audience, the cluster's issuer, a valid `exp`, and its `sub`
  (`system:serviceaccount:<namespace>:<name>`) must be on the allowlist. Add `Caller` to a handler to
  protect a route; `/v1/whoami` shows it. Nothing configured means every protected route is closed.
- `devenv.yaml` + `devenv.nix` (no Nix flakes), `Containerfile` (multi-stage, distroless runtime, base
  images pinned by digest), `deploy/deploy.cue`, `rust-toolchain.toml`, `deny.toml` (cargo-deny),
  release-please config, CODEOWNERS, `.github/renovate.json`.

Environment variables the platform sets (see platform-kit's README): `PORT`, `APP_ENV`,
`APP_VERSION`, and when `network.allowFrom` is not empty `AUTH_AUDIENCE` and `AUTH_ALLOWED_SUBJECTS`.
Your own: `LOG_LEVEL` (or `RUST_LOG`), `SHUTDOWN_DELAY_SECS`, `REQUEST_TIMEOUT_SECS`. Auth needs
`network.kubeAPI: true`, which mounts `/var/run/secrets/kube-api/{token,ca.crt}`
(`AUTH_KUBE_TOKEN_FILE`, `AUTH_KUBE_CA_FILE`, `AUTH_KUBE_API_URL`, `AUTH_ISSUER` override the defaults).

## Onboarding a new app

Members cannot create repositories; an app starts as a PR to the platform:

1. **Platform PR**: the platform team adds `{"name": "<app>", "team": "<team>"}` to `apps.json` in
   `platform-terraform` (a new team also needs `teams.json` and `platform-components` `teams/<team>.cue`).
2. **Owner applies** `github/` (the repository is created from this template, with the custom
   properties `owner-team` and `kind=app`, team access, the Environments `staging` and `prod`, rulesets)
   and `openbao/` (roles and policies `<team>-<env>-<app>`).
3. **First PR in the new repository** renames the template (see "After Use this template" below: Cargo
   package, `Containerfile` `ARG BIN`, `deploy/deploy.cue` `name` and `team`, `CODEOWNERS`, the octo-sts
   `subject`). Merge it to `main`.
4. **`register.yaml`** runs on that push (`app-register`, octo-sts identity `app-register`) and
   opens a PR in gitops: `apps/<team>/<app>/app.cue` and the registry entry. gitops-verify checks
   that `deploy.cue`'s team equals the repository's `owner-team` property; the platform team approves.
5. The first tag (release-please PR, merge) runs the normal release flow.

## After "Use this template"

1. **Name.** Rename the package in `Cargo.toml` and the crate in `src/main.rs`, `tests/*.rs`
   (`rust_app_template` → your crate), then `cargo update -w` to refresh `Cargo.lock`, and set
   `ARG BIN` in the `Containerfile` to the package name. The package name is the binary name.
2. **`deploy/deploy.cue`**: set `name` (equal to the repository name) and `team`; then what the app
   needs (`database`, `secrets`, `network`). `cue.mod/module.cue`: set the module path
   `github.com/homelab-selfuryon/<repository>@v0`.
3. **`.github/CODEOWNERS`**: your team, `@homelab-selfuryon/<team>`.
4. **`.github/chainguard/release-please.sts.yaml`**: the repository name in `subject`.
5. Platform side (nothing in this repository): see "Onboarding" above; `register.yaml` opens the gitops PR.
6. Delete what you do not need (`/v1/hello`, `/v1/whoami`, `src/auth/` if nothing calls you) and
   replace this README.

## Development

Tools come from devenv (no Nix flakes); `devenv test` is what CI runs.

```sh
devenv shell           # rust toolchain from rust-toolchain.toml, cue (0.18 pre-release), cargo-deny, treefmt, git hooks
                       # (or direnv: `use devenv`, see .envrc; or `devenv hook`)
treefmt                # format everything: rustfmt, nixfmt, taplo, cue fmt (same config as the git hook)
cargo test             # unit and HTTP tests; none needs a cluster or a database
cargo run              # PORT=8080
devenv test            # treefmt --fail-on-change, toolchain/Containerfile match, clippy -D warnings, tests, cargo deny check
docker buildx build -f Containerfile .   # the image (podman build -f Containerfile . works too)
devenv shell -- cue vet ./deploy        # schema check; `cue export ./deploy -e name --out text`
```

`cargo deny check` includes advisories and needs the network. The Containerfile build stage
(`FROM rust:<version>-slim-bookworm`) must name the Rust version of `rust-toolchain.toml`; Renovate
bumps both in one PR and `devenv test` checks it. `devenv.lock` pins the dev tools; Renovate has no
devenv manager, so refresh it with `devenv update` (a scheduled workflow is planned).

Render a manifest set locally: write `deploy/context_gen.cue` (git-ignored)

```cue
package deploy

context: {env: "staging", image: "ghcr.io/homelab-selfuryon/<app>@sha256:<64 hex>", version: "v0.1.0"}
```

and `cue export ./deploy -e manifestsYAML --out text`. `cue` resolves platform-kit from GHCR:
`CUE_REGISTRY='inline:moduleRegistries: "github.com/homelab-selfuryon": {registry: "ghcr.io/homelab-selfuryon/cue", stripPrefix: true}'` (log in with `oras login ghcr.io`
or `docker login ghcr.io` if the package is not public).

## Delivery

- **Pull request**: `ci.yaml` runs `rust-ci` (`devenv test`, `cue vet`). With the label `preview` (branches
  of this repository only), `preview.yaml` runs `app-preview`: image `pr-<n>-<sha7>` and manifests
  `pr-<n>`, signed with the preview identity; gitops creates the environment `<team>-pr-<n>` while
  the PR is open and labelled, and removes it when either goes.
- **main**: `release-please.yaml` opens a release PR (changelog, version from Conventional Commits).
  Merging it creates the tag `v<version>`. The tag is created with an octo-sts token so that it
  triggers `release.yaml`.
- **Tag `v*`**: `release.yaml` → `app-release`: image, SBOM, provenance, manifests for staging and
  prod, and a PR in gitops that moves staging (merged automatically after checks).
- **prod**: Actions → Promote → "Use workflow from": the release tag. It opens a PR in gitops that the
  team approves. A branch is refused.
- **Rollback**: revert the PR in gitops.

Commits follow [Conventional Commits](https://www.conventionalcommits.org/) (`!` for breaking changes)
and are signed.
