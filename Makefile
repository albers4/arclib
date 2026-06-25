PY_PKG_DIR 	:= python
VENV_DIR 	:= .venv
PYTHON 		:= $(VENV_DIR)/bin/python
PIP 		:= $(VENV_DIR)/bin/pip
DOCS_DIR	:= docs/source
RUST_DOCS	:= target/doc
PYTHON_DOCS	:= target/doc/python

# ----- Rename -----

OLD_NAME = arclib
NEW_NAME = arclib

.PHONY: rename-dry rename
rename-dry: clean
	find . -depth -type d -name '*$(OLD_NAME)*' -exec rename -n "s/$(OLD_NAME)/$(NEW_NAME)/g" {} \;
	rg -l --null -i '$(OLD_NAME)' | xargs -0 sed -n 's/$(OLD_NAME)/$(NEW_NAME)/gip'

rename: clean
	find . -depth -type d -name '*$(OLD_NAME)*' -exec rename "s/$(OLD_NAME)/$(NEW_NAME)/g" {} \;
	rg -l --null -i '$(OLD_NAME)' | xargs -0 sed -i 's/$(OLD_NAME)/$(NEW_NAME)/g'

# ----- Rust -----

.PHONY: rust
rust:
	cargo build --workspace --release

# ----- Python -----

.PHONY: python-venv python-dev-deps python-deps python
python-venv:
	test -d $(VENV_DIR) || python3.12 -m venv $(VENV_DIR)
	$(PIP) install --upgrade pip
	$(PIP) install build

python-dev-deps: python-venv
	$(PIP) install -r requirements-dev.txt
	cd $(PY_PKG_DIR) && ../$(PYTHON) -m maturin develop

python-deps: python-venv
	$(PIP) install -r requirements.txt

python: python-deps python-dev-deps rust
	cd $(PY_PKG_DIR) && ../$(PYTHON) -m build

# ----- Test -----

.PHONY: rust-test python-test test
rust-test:
	ARCLIB_TEST_OUTPUT=$(CURDIR)/build/test_output cargo test --release --workspace -- --no-capture

python-test: python-dev-deps
	cd $(PY_PKG_DIR) && ../$(PYTHON) -m pytest -s tests

test: rust-test python-test

# ----- Benchmark -----

.PHONY: rust-benchmark python-benchmark benchmark
rust-benchmark:
	cargo bench -p arclib

python-benchmark: python-dev-deps
	cd $(PY_PKG_DIR) && ../$(PYTHON) -m pytest benches

benchmark: rust-benchmark python-benchmark

# ----- Documentation -----

.PHONY: rust-documentation python-documentation documentation documentation-open
rust-documentation:
	cargo doc --workspace --no-deps

python-documentation: python
	$(PYTHON) -m sphinx -b html $(DOCS_DIR) $(PYTHON_DOCS)

documentation: rust-documentation python-documentation

documentation-open: documentation
	xdg-open docs/rust.html
	xdg-open docs/python.html

# ----- Format/Lint -----

.PHONY: rust-format python-format format rust-lint python-lint lint
rust-format:
	cargo fmt --all

python-format: python-dev-deps
	$(PYTHON) -m ruff format $(PY_PKG_DIR)

format: rust-format python-format

rust-lint:
	cargo clippy --workspace -- -D warnings

python-lint: python-dev-deps
	$(PYTHON) -m ruff check $(PY_PKG_DIR)

lint: rust-lint python-lint

# ----- Clean -----

.PHONY: clean
clean:
	cargo clean
	rm -rf $(VENV_DIR)
	git clean -fdX
