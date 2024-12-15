# mc

Modern file copying

## Features

- Copy file or folder
- Progress bar
- Verify with hash
- Hard link files
- Symbol link files

## Test

```console
dd if=/dev/zero of=dummy bs=2G count=10
mc dummy copied_dummy --verify
```

