VAR="${1:-integration}"
echo "Running tests ... $VAR"
SESSION_KEY=test cargo test --features integration_tests $VAR -- --test-threads=1 --nocapture
