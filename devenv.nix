{ ... }:
{
  # Everything comes from the platform-kit module (see devenv.yaml): toolchain, tools, treefmt, git
  # hooks, and `devenv test` (treefmt, toolchain/Containerfile check, clippy, tests, cargo-deny).
  # Add the app's own here, e.g. platformKit.postgres.enable = true;
}
