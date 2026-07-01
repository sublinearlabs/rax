PERF_BASE ?= main
PERF_RUNS ?= 7
PERF_WARMUPS ?= 1

.PHONY: perf-aot clean

perf-aot:
	cargo build --release --locked -p rax-perf
	./target/release/rax-perf branch-report --base $(PERF_BASE) --runs $(PERF_RUNS) --warmups $(PERF_WARMUPS)

clean:
	cargo clean
