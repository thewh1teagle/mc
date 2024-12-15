# mc

Modern file copying

## Features

- 📂 Copy file or folder
- 🔄 Progress bar
- 🔐 Verify with hash
- 🔗 Hard link files
- 🔗🔗 Symbolic link files
- 🛏️ Keep system awake while copying

## Install

See installation options in [Mc Website](https://thewh1teagle.github.io/mc)

## Usage

```console
dd if=/dev/zero of=dummy bs=2G count=10
mc dummy copied_dummy --verify
```

