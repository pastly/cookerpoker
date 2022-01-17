TESTS=$(shell find . -mindepth 1 -type d)

.PHONY: all rebuild $(TESTS)

all: $(TESTS)

rebuild:
	cargo build --bin manual-game

$(TESTS): rebuild
	@cd $@ && \
	rm -vf output.actual.txt; \
	<input.txt \
	../../../target/release/manual-game \
		--no-prompts \
		$$(cat args.txt) \
	>output.actual.txt && \
	echo $@ && \
	./test.sh output.expected.txt output.actual.txt
