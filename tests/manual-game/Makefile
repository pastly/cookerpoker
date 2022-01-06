TESTS=$(shell find . -mindepth 1 -type d)

.PHONY: all rebuild $(TESTS)

all: $(TESTS)

rebuild:
	cargo build --release --bin manual-game

$(TESTS): rebuild
	rm -vf output.actual.txt
	cd $@ && \
	<input.txt \
	../../../target/release/manual-game \
		--no-prompts \
		$$(cat args.txt) \
	>output.actual.txt && \
	echo $@ && \
	./test.sh output.expected.txt output.actual.txt
