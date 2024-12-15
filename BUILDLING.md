# Building

## Release new version

```console
git tag v0.1.0
git push --tags
```

## Test with dummay file

```console
dd if=/dev/zero of=dummy bs=1G count=5
mc dummy copied_dummy --verify
```
