VAR="${1:-integration}"
echo "Running tests ... $VAR"
cargo test --features integration_tests $VAR -- --test-threads=1 --nocapture
