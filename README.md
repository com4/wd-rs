# Warpdir

waprdir provides shortcuts for `cd` under the `wd` alias. This is a cross-shell/cross-platform drop in replacement the [zsh plugin](https://github.com/mfaerevaag/wd) by mfaerevaag (that is to say all of your warps are stored in `~/.warprc` and are compatible with the plugin.)

## Note
If you are seeing this on Github you are viewing a read-only mirror of the source code. You can view the entire repository and download releases at https://rc4.net/wd-rs/

## Source Control

To grab a copy of the repository download the latest version of [Fossil](https://fossil-scm.org) and run something like this:
```bash
mkdir ./repositories ./wd-rs
fossil clone https://rc4.net/wd-rs/ ./repositories/wd-rs.fossil
cd ./wd-rs
fossil open ../repositories/wd-rs.fossil
```

## Building and Releasing

Releases are built by running the `build_release.sh` script in `tools/release/` directory but generally this is what the script does for each target system:

```bash
emacs Cargo.toml  # update the version
fossil commit --tag <version>
rm -rf target/
cargo build --release
cd target/release/
strip warpdir
tar czf warpdir-$VERSION-$ARCH.tar.gz warpdir
sha256sum -b warpdir-$VERSION-$ARCH.tar.gz > warpdir-$VERSION-$ARCH.tar.gz.sha256

sha256sum -c warpdir-$VERSION-$ARCH.tar.gz.sha256 \
  && fossil uv add warpdir-$VERSION-$ARCH.tar.gz warpdir-$VERSION-$ARCH.sha256 \
  && fossil uv sync
```

They are then synced to the developer machine, the hash verified, gpg signed, and uploaded

```bash
fossil uv sync
mkdir releases && cd releases

fossil uv export warpdir-$VERSION-$ARCH.tar.gz warpdir-$VERSION-$ARCH.tar.gz
fossil uv export warpdir-$VERSION-$ARCH.sha256 warpdir-$VERSION-$ARCH.sha256
sha256sum -c warpdir-$VERSION-$ARCH.sha256

# If the hash matches and the archive wasn't corrupted
gpg --local-user <key> --sign --detach-sig -a --output warpdir-$VERSION-$ARCH.tar.gz.gpg warpdir-$VERSION-$ARCH.tar.gz
gpg --verify warpdir-$VERSION-$ARCH.tar.gz.gpg warpdir-$VERSION-$ARCH.tar.gz

fossil uv add *.gpg
fossil uv sync
```

Finally, the download page is updated by modifying the `release` variable in download.js, adding the new version information and removing stale entries.

```bash
fossil uv edit download.js
fossil uv rm --glob warpdir-$STALE_VERSION*
fossil uv sync
```
