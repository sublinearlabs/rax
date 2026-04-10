//! Example: Complete RISC-V to x86-64 ELF compilation pipeline
//!
//! This example demonstrates:
//! 1. Creating x86-64 bytecode using the emitter
//! 2. Generating an ELF binary file
//! 3. Writing to disk
//! 4. Attempting to execute the binary
//! 5. Verifying the results

use riscv::translate::{ElfBuilder, ElfConfig, Operand, X86Emitter, X86Register};
use std::fs;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ============================================================
    // Phase 1: Generate x86-64 bytecode
    // ============================================================

    println!("=== Phase 1: Generating x86-64 bytecode ===\n");

    let mut emitter = X86Emitter::new();

    // Linux x86-64 exit syscall: mov rax, 60; mov rdi, exit_code; syscall
    // rax = 60 (exit syscall number)
    emitter.emit_mov(
        &Operand::Immediate(60),
        &Operand::Register(X86Register::RAX),
    )?;

    println!("Generated: MOV RAX, 60 (exit syscall)");
    println!("Bytecode: {:02X?}\n", emitter.get_buffer());

    // rdi = 42 (exit code)
    emitter.emit_mov(
        &Operand::Immediate(42),
        &Operand::Register(X86Register::RDI),
    )?;

    println!("Generated: MOV RDI, 42 (exit code)");
    println!("Bytecode: {:02X?}\n", emitter.get_buffer());

    // syscall instruction (0x0F 0x05)
    emitter.emit_bytes(&[0x0F, 0x05]);

    println!("Generated: SYSCALL");
    println!("Bytecode: {:02X?}\n", emitter.get_buffer());

    // Finalize to get the bytecode
    let bytecode = emitter.finalize()?;

    println!("Total bytecode size: {} bytes\n", bytecode.len());

    // ============================================================
    // Phase 2: Generate ELF binary
    // ============================================================

    println!("=== Phase 2: Generating ELF binary ===\n");

    // Create ELF configuration
    let config = ElfConfig {
        entry_point: 0x400000,
        text_vaddr: 0x400000,
        data_vaddr: 0x600000,
        bss_vaddr: 0x601000,
        page_align: 0x1000,
    };

    println!("ELF Configuration:");
    println!("  Entry Point: 0x{:X}", config.entry_point);
    println!("  .text Address: 0x{:X}", config.text_vaddr);
    println!("  .data Address: 0x{:X}", config.data_vaddr);
    println!("  Page Alignment: 0x{:X}\n", config.page_align);

    // Create ELF builder and add bytecode
    let mut builder = ElfBuilder::with_config(config);
    builder.add_text(bytecode);

    // Build ELF binary
    let elf_binary = builder.build()?;

    println!("ELF binary generated successfully!");
    println!("ELF binary size: {} bytes\n", elf_binary.len());

    // Verify ELF magic
    if &elf_binary[0..4] == b"\x7FELF" {
        println!("✓ ELF magic number verified: {:02X?}", &elf_binary[0..4]);
    } else {
        println!("✗ ERROR: Invalid ELF magic number!");
        return Err("Invalid ELF format".into());
    }

    // ============================================================
    // Phase 3: Write to disk
    // ============================================================

    println!("\n=== Phase 3: Writing to disk ===\n");

    let output_path = "target/example_output.elf";
    fs::write(output_path, &elf_binary)?;

    // Set executable permission on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(output_path, perms)?;
    }

    println!("✓ ELF binary written to: {}\n", output_path);

    // Print file info
    let metadata = fs::metadata(output_path)?;
    println!("File size: {} bytes", metadata.len());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = metadata.permissions();
        println!("Permissions: 0o{:o}", permissions.mode());
    }

    // ============================================================
    // Phase 4: Attempt to execute the binary
    // ============================================================

    println!("\n=== Phase 4: Attempting to execute the binary ===\n");

    #[cfg(unix)]
    {
        match Command::new(output_path).output() {
            Ok(output) => {
                println!("✓ Binary executed successfully!");
                println!("Exit code: {:?}", output.status.code());
                if !output.stdout.is_empty() {
                    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                println!("✗ Failed to execute binary: {}", e);
                println!("  (This may indicate missing program headers or other ELF issues)");
            }
        }
    }

    #[cfg(not(unix))]
    {
        println!("⊘ Execution testing not supported on this platform");
    }

    // ============================================================
    // Phase 5: Diagnostic information
    // ============================================================

    println!("\n=== Phase 5: Diagnostic information ===\n");

    #[cfg(unix)]
    {
        println!("File type inspection:");
        match Command::new("file").arg(output_path).output() {
            Ok(output) => {
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
            Err(_) => println!("  (file command not available)"),
        }

        println!("Program headers:");
        match Command::new("readelf").arg("-l").arg(output_path).output() {
            Ok(output) => {
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
            Err(_) => println!("  (readelf command not available)"),
        }

        println!("Disassembly:");
        match Command::new("objdump").arg("-d").arg(output_path).output() {
            Ok(output) => {
                let disasm = String::from_utf8_lossy(&output.stdout);
                // Print first 50 lines of disassembly
                for (i, line) in disasm.lines().enumerate() {
                    if i < 50 {
                        println!("{}", line);
                    }
                }
                if disasm.lines().count() > 50 {
                    println!("  ... (output truncated)");
                }
            }
            Err(_) => println!("  (objdump command not available)"),
        }
    }

    println!("\n=== Summary ===");
    println!("✓ Bytecode generation: OK");
    println!("✓ ELF binary generation: OK");
    println!("✓ File written: {}", output_path);
    println!("\nNext steps:");
    println!("1. Check program headers with: readelf -l {}", output_path);
    println!("2. Disassemble with: objdump -d {}", output_path);
    println!("3. Inspect binary with: hexdump -C {}", output_path);

    Ok(())
}
