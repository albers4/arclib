# arclib

## Initial setup (first time only)
```
make python-venv        # Create virtual environment
make python-dev-deps    # Install development dependencies + maturing develop
make rust               # Build Rust library
```

## Complete Build (most common)
```
make python             # This build Rust + installs deps + builds Python package
```

### Testing
```
make test               # Runs both Rust and Python tests
# Or individually:
make rust-test          # Only Rust tests
make python-test        # Only Python tests
```

## Benchmarking
```
make benchmark          # Runs both Rust and Python benchmarks
# Or individually:
make rust-benchmark     # Only Rust benchmark
make python-benchmark   # Only Python benchmark
```

## Documentation
```
make documentation      # Generate both Rust and Python docs
make documentation-open # Generate and open in browser
```

## Code Quality
```
make format             # Format both Rust and Python code
make lint               # Lint both Rust and python code
```

## Cleaning up
```
make clean              # Remove all build artifacts and virtual environment
```

## Typical full development cycle:
```
# Initial setup
make python-dev-deps

# Make code changes, then:

make format
make lint
make test
make benchmark

# Generate docs before commit
make documentation
```

## Rename file content and folders
Set the correct OLD_NAME and NEW_NAME in the Makefile.
```
make rename-dry         # Dry run to review changes
make rename             # Rename all folders and occurences in files
```


## License

This library is licensed under LGPL-2.1-or-later.
See [LICENSE](LICENSE) for details.

Copyright (c) 2026 ARC (Applied Research & Computation)

