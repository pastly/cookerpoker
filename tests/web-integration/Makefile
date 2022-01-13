TESTS:=$(shell find . -mindepth 1 -type d)

.PHONY: all rebuild $(TESTS)

all: $(TESTS)

rebuild:
	cargo build --bin poker-server

ADDRESS:=127.0.0.1
# Incremented by every test so that they each use a unique port and this won't
# be the reason they can't run in parallel.
PORT:=20000
# Absolute path to the database migration directory.
MIGRATION_DIR:=$(shell pwd)/../../poker-server/migrations
# Absolute path to the static file directory
STATIC_DIR:=$(shell pwd)/../../poker-server/static
TEMPLATE_DIR:=$(shell pwd)/../../poker-server/templates
# Relative path to the file that shall contain the sqlite database for a specific test
DATABASE_URL:=test.db
# Relative path to the (optional) file that contains SQL statements that should
# be executed on the database before the test is run.
INPUT_SQL:=input.sql

$(TESTS): rebuild
	$(eval PORT=$(shell echo $$(($(PORT)+1))))
	@echo ---------- $@ ----------
	@cd $@ && \
	rm -vf \
		$(DATABASE_URL) \
		static \
		templates \
		*.log \
		*.pid
	@cd $@ && \
	ln -s $(STATIC_DIR) static && \
	ln -s $(TEMPLATE_DIR) templates

	@#cd $@ && ls -lhs

	@cd $@ && \
	diesel migration run \
		--migration-dir $(MIGRATION_DIR) \
		--database-url $(DATABASE_URL)

	@cd $@ && \
	[ -f $(INPUT_SQL) ] && sqlite3 $(DATABASE_URL) < $(INPUT_SQL) || echo No $(INPUT_SQL) to execute
	
	cd $@ && \
	ROCKET_ADDRESS=$(ADDRESS) \
	ROCKET_PORT=$(PORT) \
	ROCKET_TEMP_DIR=$(shell pwd)/$@ \
	ROCKET_DATABASES={sqlite={url=$(DATABASE_URL)}} \
	../start-server.sh

	sleep 5

	cd $@ && \
	../stop-server.sh
