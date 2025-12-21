# Third-party files

This repository can optionally use sample data from upstream Spine runtimes for smoke tests and demos.

By default, upstream files are **not** committed to this repository. Use the import script to populate them locally:

```sh
./scripts/import_spine_runtimes_examples.zsh --mode json --scope tests
```

Or fetch + import in one step (downloads `spine-runtimes` into `.cache/`):

```sh
python3 ./scripts/fetch_spine_runtimes_examples.py --mode json --scope tests
```

Then run the optional smoke tests:

```sh
cargo test -p spine2d --features json,upstream-smoke
```

See `assets/spine-runtimes/` for the local import destination.
