# mc

Modern file copying

## Features

- 📂 Copy file or folder
- 🔄 Progress bar
- 🔐 Verify with hash
- 🔗 Hard link files
- 🔗 Symbolic link files
- ⚡ ~Faster than Finder or Explorer~
- 🛏️ Keep system awake while copying
- 🔄 Auto update with `mc-cli-update` command

## Install

See installation options in [Mc Website](https://thewh1teagle.github.io/mc/) or in Github [Releases](https://github.com/thewh1teagle/mc/releases/latest)

## Usage

```console
dd if=/dev/zero of=dummy bs=2G count=10
mc dummy copied_dummy --verify
```
