# Warpdir

waprdir provides shortcuts for `cd` under the `wd` alias. This is a cross-shell/cross-platform drop in replacement the [zsh plugin](https://github.com/mfaerevaag/wd) by mfaerevaag (that is to say all of your warps are stored in `~/.warprc` and are compatible with the plugin.)

## Installing

Download a binary for your system from the [downloads page](https://rc4.net/wd-rs/uv/download.html) and put `warpdir` somewhere on your path.

### Bash

Add the following to your `.bashrc`

```bash
# Enable warpdir (https://github.com/com4/wd-rs/)
WARPDIR=`whereis -b warpdir | awk {'print $2'}`
if [ -x $WARPDIR ]; then
    eval "$(warpdir hook bash)"
fi
```

To enable it on existing termianls you can type `source ~/.bashrc`.

## Usage

```bash
cd ~/some/nested/directory/thats/tedious/to/tab/complete/mydir
wd add mydir
# Successfully added mydir -> ~/some/nested/directory/thats/tedious/to/tab/complete/mydir
cd ~
wd mydir
pwd
# ~/some/nested/directory/thats/tedious/to/tab/complete/mydir
```

To remove a warp

```bash
wd rm mydir
```

To list your warps

```bash
wd list
```

Also check out `wd help` for more information
