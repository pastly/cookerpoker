TESTS:=$(shell find . -mindepth 1 -type d)

.PHONY: all deps clean rebuild $(TESTS)

all: $(TESTS)

rebuild:

clean:
	cat */*.pid | xargs --no-run-if-empty kill || /bin/true
	rm -fv */*.pid
	rm -fv */$(DATABASE_URL)

deps:
	@which sqlite3 || echo missing sqlite3 binary

PWD:=$(shell pwd)
ADDRESS:=127.0.0.1
# Incremented by every test so that they each use a unique port and this won't
# be the reason they can't run in parallel.
PORT:=20000
VENV:=$(PWD)/../../venv
PYTHON3:=$(VENV)/bin/python3
MANAGE_PY:=$(PWD)/../../poker-server/manage.py
# Relative path to the file that shall contain the sqlite database for a
# specific test
DATABASE_URL:=test.db
## Absolute path to the database migration directory.
#MIGRATION_DIR:=$(shell pwd)/../../poker-server/migrations
## Absolute path to the static file directory
#STATIC_DIR:=$(shell pwd)/../../poker-server/static
## Absolute path to the templates directory
#TEMPLATE_DIR:=$(shell pwd)/../../poker-server/templates
## Absolute path to the poker-client crate
#POKER_CLIENT_CRATE:=$(shell pwd)/../../poker-client
# Relative path to the (optional) file that contains SQL statements that should
# be executed on the database before the test is run.
INPUT_SQL:=input.sql


$(TESTS): clean rebuild deps
	$(eval PORT=$(shell echo $$(($(PORT)+1))))
	@echo ---------- $@ ----------
	export DATABASE_URL=$@/$(DATABASE_URL) && \
		$(PYTHON3) $(MANAGE_PY) migrate
	[ -f $@/$(INPUT_SQL) ] && sqlite3 $@/$(DATABASE_URL) <$@/$(INPUT_SQL)
	cd $@ && \
	export DATABASE_URL=$(DATABASE_URL) && \
	PYTHON3=$(PYTHON3) \
	MANAGE_PY=$(MANAGE_PY) \
	PORT=$(PORT) \
	../start-server.sh || ../stop-server.sh
