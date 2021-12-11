{
  description = "A flake to run pazi's integ tests in a reproducible environment";

  outputs = { self, nixpkgs }:
  let
    pkgs = import nixpkgs {
      system = "x86_64-linux";
      config = {};
    };
  in
  {
    # Run tests with a version controlled bash and zsh
    # Eventually we should also version control everything in ./testbins too,
    # but for now just rely on the existing mechanism. They're pinned, it
    # works, we can update that separately.
    # This, notably, pins zsh to a version that works with our vte harness.
    runTests = pkgs.writeShellScriptBin "pazi-integ-tests" ''
      die() {
        1>&2 echo "$@"
        exit 1
      }
      set -eux

      [[ -e ../target/release/pazi ]] || die "No ../target/release/pazi file to run"
      make jump

      CARGO_BIN=$(which cargo)
      export PATH="${pkgs.zsh}/bin:${pkgs.coreutils}/bin:${pkgs.bashInteractive}/bin:${pkgs.fish}/bin:$PATH"
      sudo -E PAZI_TEST_CGROUP=true PATH="$PATH" $CARGO_BIN +nightly test --features=nightly -- --test-threads=1 "$@"
    '';

  };
}
