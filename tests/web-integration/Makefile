TESTS:=$(shell find . -mindepth 1 -type d)

.PHONY: all deps clean rebuild $(TESTS)

all: $(TESTS)

rebuild:
	cargo build --bin poker-server
	cd ../../poker-client && cargo build --features web-integration-tests --tests

clean:
	@pkill poker-server || /bin/true


deps:
	@which diesel || cargo install diesel_cli --no-default-features --features sqlite
	#@which wasm-pack || cargo install wasm-pack

ADDRESS:=127.0.0.1
# Incremented by every test so that they each use a unique port and this won't
# be the reason they can't run in parallel.
PORT:=20000
# Absolute path to the database migration directory.
MIGRATION_DIR:=$(shell pwd)/../../poker-server/migrations
# Absolute path to the static file directory
STATIC_DIR:=$(shell pwd)/../../poker-server/static
# Absolute path to the templates directory
TEMPLATE_DIR:=$(shell pwd)/../../poker-server/templates
# Absolute path to the poker-client crate
POKER_CLIENT_CRATE:=$(shell pwd)/../../poker-client
# Relative path to the file that shall contain the sqlite database for a specific test
DATABASE_URL:=test.db
# Relative path to the (optional) file that contains SQL statements that should
# be executed on the database before the test is run.
INPUT_SQL:=input.sql
# Arguments to pass to `cargo test` command. A use case: set this as an environment variable to
# only run a specific subtest. For example, for test foo that has many tests in foo.rs, once of
# which is bar, the command line "make foo CARGO_TEST_ARGS=bar" would run the foo test and only the
# bar test within it.
CARGO_TEST_ARGS:=

$(TESTS): clean rebuild deps
	$(eval PORT=$(shell echo $$(($(PORT)+1))))
	@echo ---------- $@ ----------
	@rm -vf \
		$@/$(DATABASE_URL) \
		$@/static \
		$@/templates \
		$@/*.log \
		$@/*.pid

	@cd $@ && \
	ln -s $(STATIC_DIR) static && \
	ln -s $(TEMPLATE_DIR) templates

	@cd $@ && \
	diesel migration run \
		--migration-dir $(MIGRATION_DIR) \
		--database-url $(DATABASE_URL)

	@cd $@ && \
	[ -f $(INPUT_SQL) ] && sqlite3 $(DATABASE_URL) < $(INPUT_SQL) || echo No $(INPUT_SQL) to execute
	
	@cd $@ && \
	ROCKET_ADDRESS=$(ADDRESS) \
	ROCKET_PORT=$(PORT) \
	ROCKET_TEMP_DIR=$(shell pwd)/$@ \
	ROCKET_DATABASES={sqlite={url=$(DATABASE_URL)}} \
	../start-server.sh

	@cd $(POKER_CLIENT_CRATE) && \
	ROCKET_ADDRESS=$(ADDRESS) \
	ROCKET_PORT=$(PORT) \
	cargo test --features web-integration-tests --test $@ $(CARGO_TEST_ARGS)

	@cd $@ && \
	../stop-server.sh
