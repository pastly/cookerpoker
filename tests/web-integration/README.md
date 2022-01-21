# Web integration tests

This directory contains integration tests that mimic a poker-client and
interact with poker-server.

Tests are identified by their directory names (e.g. "loginout") and have a
corresponding `.rs` file in poker-client/tests` that is executed.

Before each test, a new environment is created for poker-server, database
migrations run, and any input SQL executed. Then poker-server is started.
Finally the `.rs` file is compiled and executed.

**Note**: the server's environment and database is setup once before all tests
inside the client `.rs` file are executed. If each one needs a clean
environment, each one needs its own test directory. That's how it is at the
time of writing.

## Running the tests

To run all tests, execute `make`.

To execute a specific test, say loginout, run `make loginout`.

If `foo()` is one of the tests in `loginout.rs` and you want to run just it,
run `make loginout CARGO_TEST_ARGS=foo`.

## Creating a new test

Follow the lead of an existing test. Loginout is simple and expected to stay
that way.

- Create a new directory in tests/web-integration (call it "foo").

- (Optional) Create foo/input.sql for any sql statements you'd like to execute
  before the test is started.

- Create a "foo.rs" in poker-client/tests.

- As the first line in foo.rs, use `#![cfg(feature = "web-integration-tests")]`
  so it isn't normally compiled.

- Add your tests to foo.rs. You probably want to use `#[tokio::test]` for them
  if you want to use the async http functions.
