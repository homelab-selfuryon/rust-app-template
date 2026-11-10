# rust-app-template

Template of a Rust app on the homelab-selfuryon platform. Contract with the rest of the org:
`meta/delivery.md`; card `meta/rust-app-template.md`. Apps (`notes-api`, `notes-stats`) are
copies of this repository: a fix to the common parts belongs here and in them.

## Layout

```
src/main.rs        tracing, config, verifier, serve with graceful shutdown
src/lib.rs         modules; src/config.rs env config; src/secrets.rs files + secrecy
src/auth/          verifier (cache, rotation, rate limit), jwks (kube-apiserver), Caller extractor
src/routes.rs      /healthz /readyz and the example routes
tests/             auth.rs (verifier), http.rs (router), kube_jwks.rs (apiserver stand-in), common/
deploy/deploy.cue  the app contract (platform-kit #App)
devenv.yaml/.nix   tools: the platform-kit devenv module (toolchain, treefmt, git hooks, `devenv test`); no Nix flakes
Containerfile      multi-stage image build, base images pinned by digest; .dockerignore
.github/workflows/ thin callers of github-workflows: ci, release-please, release, promote, register
```

## Commands

```sh
devenv shell -- cargo test                         # no database needed
devenv shell -- treefmt                            # format (devenv test uses --fail-on-change)
devenv test                                        # the CI contract: treefmt, toolchain/Containerfile match,
                                                   # clippy -D warnings, cargo test, cargo deny check
devenv shell -- cue vet ./deploy
docker buildx build -f Containerfile .             # or: podman build -f Containerfile .
devenv shell -- actionlint && devenv shell -- zizmor --offline .github/workflows
```

devenv.lock is not committed until platform-kit v0.1.0 is published (the import needs it); then
`devenv update` and commit; refresh it the same way. Against an unpublished platform-kit:
`devenv -o platform-kit path:../platform-kit test`.

## Rules

- Secrets are files (`secrets.rs`, `SecretString`), never env variables, never logged.
- Deny by default: a route is public only when it says so; protected routes take `Caller`.
  No `unwrap`/`expect` outside tests (clippy lint), no `unsafe`.
- Logs: JSON on stdout through `tracing`; no request bodies, no tokens.
- Tests need no cluster and no database; anything that does is `#[ignore]` with the way to run it.
- Workflows are thin: `uses: homelab-selfuryon/github-workflows/...@v1.0.0` (Renovate pins SHAs),
  top-level `permissions: {}`, nothing with `${{ }}` inside `run:`. zizmor and actionlint stay clean.
- Conventional Commits, `!` for breaking changes, signed commits.
