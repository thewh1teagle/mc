# mc

Modern file copying

## Features

- 📂 Copy file or folder
- 🔄 Progress bar
- 🔐 Verify with hash
- 🔗 Hard link files
- 🔗 Symbolic link files
- 🔗 Reflink link files
- ⚡ Faster than Finder or Explorer
- 🛏️ Keep system awake while copying
- 🔄 Auto update with command (`mc-cli-update`)
- 💻 Cross platform: Windows / macOS / Linux

## Install

See installation options in [Mc Website](https://thewh1teagle.github.io/mc/) or in Github [Releases](https://github.com/thewh1teagle/mc/releases/latest)

## Usage

See `--help`

<details>

<summary>Details</summary>

```console
Copies files or directories with options for recursion and overwriting.

Usage: mc [OPTIONS] <SOURCE>... <DESTINATION>

Arguments:
  <SOURCE>...    Source file or directory to copy
  <DESTINATION>  Destination file or directory

Options:
  -f, --force               Overwrite destination if it exists
      --hard-link           Hard link file
      --symlink             Symbol link file
      --reflink             Ref link file Similar to hardlink except modify one doesn't affect the other
      --verify              Verify hash of folder / file once copied
      --no-progress         Disable progress bar
      --no-keep-awake       Disable keep system awake while copy
      --keep-display-awake  Keep display awake while copy
  -h, --help                Print help
```

</details>
