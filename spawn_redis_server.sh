exec cargo run \
    --quiet \
    --release \
    --manifest-path $(dirname $0)/Cargo.toml \
    -- "$@"
