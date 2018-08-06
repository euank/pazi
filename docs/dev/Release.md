## Creating a release

1. Write an appropriate changelog
1. Bump the version in Cargo.toml
1. Bump the version in tests:
    ```sh
    $ cd tests
    $ cargo update -p pazi
    ```
1. Create PR with the release commit with the message 'Release vX.Y.Z'. Optionally include `:tada:` and `:zap:` to taste.
1. Create a release build of pazi:
    ```sh
    $ PR_NUM=123 # the github pr number that includes the release commit
    $ RELEASE_COMMIT=abcdef123 # the commit hash of the release commit
    $ V=x.y.z
    $ mkdir out-v$V
    $ docker pull rust:latest
    $ docker run -i -v $(pwd)/out-v${V}:/out rust:latest <<EOF
      set -ex
      apt-get update && apt-get install -y musl-dev musl-tools
      rustup target add x86_64-unknown-linux-musl
      git clone https://github.com/euank/pazi.git
      cd pazi
      git fetch origin pull/${PR_NUM}/head:release
      git checkout release
      cargo build --target x86_64-unknown-linux-musl --release

      cp target/x86_64-unknown-linux-musl/release/pazi /out/pazi-x86_64-unknown-linux-musl
    EOF
    ```
1. Sign the release build of pazi:
    ```sh
    $ cd out-vX.Y.Z
    $ gpg2 --sign --armour --detach pazi-x86_64-unknown-linux-musl
    $ gpg2 --verify pazi-x86_64-unknown-linux-musl.asc
    ```
1. Sign the commit
    ```sh
    $ git checkout $RELEASE_COMMIT
    $ git tag -m v$V --sign v$V
    $ git tag --verify v$V
    ```
1. Push the release PR to master, including the tag
    ```sh
    $ git push --tags -u origin HEAD:master
    ```
1. Create a release with the above created artifacts pointing at the tag
1. Release it on crates.io
    ```sh
    $ docker run -it rust
    $ git clone https://github.com/euank/pazi
    $ cd pazi && git checkout v$V
    # Visit https://crates.io/me and create a token
    $ cargo login $token
    $ cargo package
    $ cargo publish
    ```
