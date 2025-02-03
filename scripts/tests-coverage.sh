
### NOTE: first we need to install grcov and llvm-tools-preview for test coverage
# cargo install grcov
# rustup component add llvm-tools-preview



# clean project
cargo clean

# clean test coverage profile previous files
rm -f *.profraw

# build with coverage flags, cmd run by cargo test so skip it
#RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="aurora-cli-%p-%m.profraw" cargo build

# Build test code, without running tests 
#RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="aurora-cli-%p-%m.profraw" cargo build --tests

# run integration tests only
#RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="aurora-cli-%p-%m.profraw" cargo test --test simple_tests -- --show-output --test-threads=1
#RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="aurora-cli-%p-%m.profraw" cargo test --test '*' -- --show-output --test-threads=1
#RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="aurora-cli-%p-%m.profraw" cargo test --test simple_tests simple_silo_tests -- --show-output --test-threads=1

# run all tests
RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="aurora-cli-%p-%m.profraw" cargo test -- --show-output --test-threads=1

# generate coverage report as html
grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing --ignore "*cargo*" -o ./coverage/html

# generate coverage report as lcov
grcov . --binary-path ./target/debug/ -s . -t lcov --branch --ignore-not-existing --ignore "*cargo*" -o ./coverage/tests.lcov
