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
	cargo run -p riscv --example echo_$* --release > baseline.txt
	cargo run -p riscv --example fib_$* --release >> baseline.txt
	cargo run -p riscv --example exec_block_$* --release >> baseline.txt
	cat baseline.txt

# usage:
# 	make compare-gc
# 	make compare-ima
# 	make compare-imac
compare-%:
	cargo run -p riscv --example echo_$* --release > compare.txt
	cargo run -p riscv --example fib_$* --release >> compare.txt
	cargo run -p riscv --example exec_block_$* --release >> compare.txt
	cat baseline.txt
	cat compare.txt

clean:
	cargo clean
