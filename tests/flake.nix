{
  description = "A flake to run pazi's integ tests in a reproducible environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }:
    let
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [
          (import ./nix/overlay.nix)
        ];
        config = { };
      };
      runTestsShared = ''
        die() {
          1>&2 echo "$@"
          exit 1
        }
        set -eux

        [[ -e ../target/release/pazi ]] || die "No ../target/release/pazi file to run"

        # Setup other binaries we test, communicated to the integ tests via env var
        export JUMP_BIN=${pkgs.jump}/bin/jump
        export AUTOJUMP_BIN=${pkgs.autojump}/bin/autojump
        export AUTOJUMP_HOOKS=${pkgs.autojump}/share/autojump/
        export FASD_BIN=${pkgs.fasd}/bin/fasd
        export RUPA_Z_BIN=${pkgs.rupa_z}/share/z.sh
        export ZOXIDE_BIN=${pkgs.zoxide}/bin/zoxide

        CARGO_BIN=$(which cargo)
        export PATH="${pkgs.zsh}/bin:${pkgs.coreutils}/bin:${pkgs.bashInteractive}/bin:${pkgs.fish}/bin:$PATH"
      '';
    in
    {
      # Run tests with a version controlled bash and zsh
      # Eventually we should also version control everything in ./testbins too,
      # but for now just rely on the existing mechanism. They're pinned, it
      # works, we can update that separately.
      # This, notably, pins zsh to a version that works with our vte harness.
      runTests = pkgs.writeShellScriptBin "pazi-integ-tests" ''
        ${runTestsShared}
        cargo test -- --test-threads=1 "$@"
      '';
      runTestsRoot = pkgs.writeShellScriptBin "pazi-integ-tests" ''
        ${runTestsShared}
        sudo -E PAZI_TEST_CGROUP=true PATH="$PATH" $CARGO_BIN +nightly test --features=nightly -- --test-threads=1 "$@"
      '';

      runBenches = pkgs.writeShellScriptBin "pazi-integ-benches" ''
        ${runTestsShared}
        sudo -E PAZI_TEST_CGROUP=true PATH="$PATH" $CARGO_BIN +nightly bench --features=nightly "$@"
      '';

    };
}
