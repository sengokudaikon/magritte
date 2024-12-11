# Define variables for configuration
VERSION := "0.1.0"
NIGHTLY_TOOLCHAIN := "+nightly"

# Utility tasks
cleanup:
    @for dir in */; do \
        (cd "$$dir" && cargo clean); \
    done

update-version version=VERSION:
    # Update versions inside crates
    sed -i 's/^version.*$/version = "'{{version}}'"/' magritte_query/Cargo.toml
    sed -i 's/^version.*$/version = "'{{version}}'"/' magritte_macros/Cargo.toml
    sed -i 's/^version.*$/version = "'{{version}}'"/' magritte/Cargo.toml
    sed -i 's/^version.*$/version = "'{{version}}'"/' magritte_migrations/Cargo.toml

    # Update dependencies in `magritte` crate
    sed -i 's/^magritte_macros [^,]*,/magritte_macros = { version = "'~{{version}}'",/' magritte/Cargo.toml

    # Update dependencies in `magritte_migrations`
    sed -i 's/^magritte_macros [^,]*,/magritte_macros = { version = "'~{{version}}'",/' magritte_migrations/Cargo.toml
    sed -i 's/^magritte [^,]*,/magritte = { version = "'~{{version}}'",/' magritte_migrations/Cargo.toml

    git commit -am "Bump version to {{version}}"

publish:
    # Publish crates in the correct order
    (cd magritte_query && cargo publish)
    (cd magritte_macros && cargo publish)
    (cd magritte_migrations && cargo publish)
    cargo publish

generate-readme:
    # Generate README.md from src/lib.rs (requires cargo-readme to be installed)
    @cargo readme --no-badges --no-indent-headings --no-license --no-template --no-title > README.md

format:
    # Format using nightly cargo
    TARGETS=("Cargo.toml"
             "magritte_query/Cargo.toml"
             "magritte_macros/Cargo.toml"
             "magritte_migrations/Cargo.toml")

    @for target in ${TARGETS[@]}; do \
        echo "Formatting $${target}"; \
        cargo {{NIGHTLY_TOOLCHAIN}} fmt --manifest-path "$${target}" --all; \
    done

    # Format examples
    EXAMPLES=$(find examples -type f -name 'Cargo.toml')
    @for example in $${EXAMPLES[@]}; do \
        echo "Formatting $${example}"; \
        cargo {{NIGHTLY_TOOLCHAIN}} fmt --manifest-path "$${example}" --all; \
    done

    slmd COMMUNITY.md -oi

clippy-fix:
    # Run clippy fixes
    TARGETS=("Cargo.toml"
             "magritte_query/Cargo.toml"
             "magritte_macros/Cargo.toml"
             "magritte_migrations/Cargo.toml")

    @for target in ${TARGETS[@]}; do \
        echo "Running clippy fixes on $${target}"; \
        cargo clippy --manifest-path "$${target}" --fix --allow-dirty --allow-staged; \
    done

    # Run clippy on examples
    EXAMPLES=$(find examples -type f -name 'Cargo.toml')
    @for example in $${EXAMPLES[@]}; do \
        echo "Running clippy fixes on $${example}"; \
        cargo clippy --manifest-path "$${example}" --fix --allow-dirty --allow-staged; \
    done

# Example usage of the `justfile`:
# - To update version: `just update-version version=1.2.3`
# - To clean directories: `just cleanup`
# - To publish crates: `just publish`
# - To format code: `just format`
# - To fix clippy warnings: `just clippy-fix`
# - To generate README: `just generate-readme`