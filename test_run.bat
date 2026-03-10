cd "H:\programing language\axon"
echo "=== CARGO CHECK ===" > test_results.log 2>&1
cargo check 2>&1 >> test_results.log
echo "=== LIBRARY TESTS ===" >> test_results.log 2>&1
cargo test --lib 2>&1 >> test_results.log
echo "=== E2E TESTS ===" >> test_results.log 2>&1
cargo test --test e2e_tests 2>&1 >> test_results.log
type test_results.log
