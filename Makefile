.PHONY: all echo_gc echo_ima echo_imac fib_gc fib_ima fib_imac exec_block_gc exec_block_ima exec_block_imac clean

all-examples: echo_gc echo_ima echo_imac fib_gc fib_ima fib_imac exec_block_gc exec_block_ima exec_block_imac

echo_gc:
	cargo run -p riscv --example echo_gc --release

echo_ima:
	cargo run -p riscv --example echo_ima --release

echo_imac:
	cargo run -p riscv --example echo_imac --release

fib_gc:
	cargo run -p riscv --example fib_gc --release

fib_ima:
	cargo run -p riscv --example fib_ima --release

fib_imac:
	cargo run -p riscv --example fib_imac --release

exec_block_gc:
	cargo run -p riscv --example exec_block_gc --release

exec_block_ima:
	cargo run -p riscv --example exec_block_ima --release

exec_block_imac:
	cargo run -p riscv --example exec_block_imac --release

# usage:
# 	make baseline-gc
# 	make baseline-ima
# 	make baseline-imac
baseline-%:
	PERF=1 cargo run -p riscv --example echo_$* --release | grep perf > baseline.txt
	PERF=1 cargo run -p riscv --example fib_$* --release | grep perf >> baseline.txt
	PERF=1 cargo run -p riscv --example exec_block_$* --release | grep perf >> baseline.txt

# usage:
# 	make compare-gc
# 	make compare-ima
# 	make compare-imac
compare-%:
	PERF=1 cargo run -p riscv --example echo_$* --release | grep perf > compare.txt
	PERF=1 cargo run -p riscv --example fib_$* --release | grep perf >> compare.txt
	PERF=1 cargo run -p riscv --example exec_block_$* --release | grep perf >> compare.txt

gen_report:
	rustc perf/report.rs -o perf/report && ./perf/report > report.txt && rm ./perf/report && cat report.txt

report-%:
	rustc perf/driver.rs -o perf/driver && ./perf/driver $* && rm ./perf/driver

clean:
	cargo clean
