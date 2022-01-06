TESTS=$(shell find . -mindepth 1 -type d)

.PHONY: all $(TESTS)

all: $(TESTS)

$(TESTS):
	rm -vf output.actual.txt
	cd $@ && \
	<input.txt \
	cargo run --release --bin manual-game -- \
		--no-prompts \
		--seed $$(cat seed.txt) \
	>output.actual.txt && \
	echo $@ && \
	./test.sh output.expected.txt output.actual.txt
