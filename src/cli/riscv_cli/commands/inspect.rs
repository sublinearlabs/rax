//! RISC-V inspect command - ELF binary structure analysis

use crate::cli::common::{check_file_exists, print_header, print_info, CliError, CliResult};
use colored::*;
use elf::{
    abi::{EM_RISCV, ET_EXEC},
    endian::LittleEndian,
    file::Class,
    ElfBytes,
};
use std::fs;

/// ELF inspection result data
#[derive(Debug)]
pub struct InspectResult {
    pub file_size: u64,
    pub entry_point: u64,
    pub num_sections: usize,
    pub num_segments: usize,
    pub architecture: String,
    pub sections: Vec<SectionInfo>,
    pub segments: Vec<SegmentInfo>,
}

#[derive(Debug, Clone)]
pub struct SectionInfo {
    pub name: String,
    pub addr: u64,
    pub size: u64,
    pub offset: u64,
}

#[derive(Debug, Clone)]
pub struct SegmentInfo {
    pub ptype: String,
    pub offset: u64,
    pub vaddr: u64,
    pub filesz: u64,
    pub memsz: u64,
}

/// Execute the inspect command
pub fn execute_inspect(binary: &str, format: &str) -> CliResult<()> {
    print_header("RISC-V CLI - ELF Binary Inspection");

    // Check file exists
    check_file_exists(binary)?;
    print_info(&format!("Loading ELF: {}", binary));

    // Read and parse ELF
    let bytes = fs::read(binary)
        .map_err(|e| CliError::new(format!("Failed to read file '{}': {}", binary, e)))?;

    let file = ElfBytes::<LittleEndian>::minimal_parse(&bytes)
        .map_err(|e| CliError::new(format!("Failed to parse ELF: {:?}", e)))?;

    let ehdr = file.ehdr;

    // Validate this is a valid RISC-V ELF
    if ehdr.class != Class::ELF64 {
        return Err(CliError::new(
            "Only 64-bit ELF files are supported".to_string(),
        ));
    }
    if ehdr.e_machine != EM_RISCV {
        return Err(CliError::new(format!(
            "Not a RISC-V ELF file (machine type: {})",
            ehdr.e_machine
        )));
    }
    if ehdr.e_type != ET_EXEC {
        return Err(CliError::new(
            "Only executable ELF files are supported".to_string(),
        ));
    }

    // Collect segment information
    let mut segments = Vec::new();
    if let Some(segs) = file.segments() {
        for seg in segs.iter() {
            segments.push(SegmentInfo {
                ptype: format_segment_type(seg.p_type),
                offset: seg.p_offset,
                vaddr: seg.p_vaddr,
                filesz: seg.p_filesz,
                memsz: seg.p_memsz,
            });
        }
    }

    // Collect section information (basic - no names without string table parsing)
    let sections = Vec::new();
    // For now, we'll skip detailed section parsing as the elf crate requires
    // string table lookup that requires more complex API usage

    let result = InspectResult {
        file_size: bytes.len() as u64,
        entry_point: ehdr.e_entry,
        num_sections: sections.len(),
        num_segments: segments.len(),
        architecture: "RISC-V 64-bit".to_string(),
        sections,
        segments,
    };

    // Display results based on format
    match format {
        "text" => format_inspect_text(&result)?,
        "json" => format_inspect_json(&result)?,
        "csv" => format_inspect_csv(&result)?,
        _ => {
            return Err(CliError::new(format!(
                "Unknown output format: '{}'. Use: text, json, csv",
                format
            )));
        }
    }

    Ok(())
}

/// Format segment type as human-readable string
fn format_segment_type(ptype: u32) -> String {
    match ptype {
        0 => "PT_NULL".to_string(),
        1 => "PT_LOAD".to_string(),
        2 => "PT_DYNAMIC".to_string(),
        3 => "PT_INTERP".to_string(),
        4 => "PT_NOTE".to_string(),
        5 => "PT_SHLIB".to_string(),
        6 => "PT_PHDR".to_string(),
        7 => "PT_TLS".to_string(),
        _ => format!("PT_UNKNOWN({})", ptype),
    }
}

/// Format inspect output as human-readable text
fn format_inspect_text(result: &InspectResult) -> CliResult<()> {
    println!("\n{}", "ELF Header Information".bold());
    println!("{}", "-".repeat(80));
    println!("  Architecture:       {}", result.architecture);
    println!("  File Size:          {} bytes", result.file_size);
    println!("  Entry Point:        0x{:x}", result.entry_point);
    println!("  Sections:           {}", result.num_sections);
    println!("  Segments:           {}", result.num_segments);
    println!();

    if !result.segments.is_empty() {
        println!("{}", "Program Headers (Segments)".bold());
        println!("{}", "-".repeat(80));
        println!(
            "{:<16} {:<12} {:<16} {:<10} {:<10}",
            "Type", "Offset", "VAddr", "FileSize", "MemSize"
        );
        println!("{}", "-".repeat(80));
        for seg in &result.segments {
            println!(
                "{:<16} 0x{:<10x} 0x{:<14x} {:<10} {:<10}",
                seg.ptype, seg.offset, seg.vaddr, seg.filesz, seg.memsz
            );
        }
        println!();
    }

    if !result.sections.is_empty() {
        println!("{}", "Sections".bold());
        println!("{}", "-".repeat(80));
        println!(
            "{:<32} {:<16} {:<10} {:<12}",
            "Name", "Address", "Size", "Offset"
        );
        println!("{}", "-".repeat(80));
        for sec in &result.sections {
            println!(
                "{:<32} 0x{:<14x} {:<10} 0x{:<10x}",
                sec.name, sec.addr, sec.size, sec.offset
            );
        }
    }

    println!();
    Ok(())
}

/// Format inspect output as JSON
fn format_inspect_json(result: &InspectResult) -> CliResult<()> {
    let segments_json: Vec<_> = result
        .segments
        .iter()
        .map(|s| {
            serde_json::json!({
                "type": s.ptype,
                "offset": format!("0x{:x}", s.offset),
                "vaddr": format!("0x{:x}", s.vaddr),
                "filesz": s.filesz,
                "memsz": s.memsz,
            })
        })
        .collect();

    let sections_json: Vec<_> = result
        .sections
        .iter()
        .map(|s| {
            serde_json::json!({
                "name": s.name,
                "addr": format!("0x{:x}", s.addr),
                "size": s.size,
                "offset": format!("0x{:x}", s.offset),
            })
        })
        .collect();

    let json = serde_json::json!({
        "elf_header": {
            "architecture": result.architecture,
            "file_size": result.file_size,
            "entry_point": format!("0x{:x}", result.entry_point),
            "num_sections": result.num_sections,
            "num_segments": result.num_segments,
        },
        "segments": segments_json,
        "sections": sections_json,
    });

    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    Ok(())
}

/// Format inspect output as CSV
fn format_inspect_csv(result: &InspectResult) -> CliResult<()> {
    println!("# ELF Header");
    println!("architecture,{}", result.architecture);
    println!("file_size,{}", result.file_size);
    println!("entry_point,0x{:x}", result.entry_point);
    println!("num_sections,{}", result.num_sections);
    println!("num_segments,{}", result.num_segments);
    println!();

    if !result.segments.is_empty() {
        println!("# Program Headers");
        println!("type,offset,vaddr,filesz,memsz");
        for seg in &result.segments {
            println!(
                "{},0x{:x},0x{:x},{},{}",
                seg.ptype, seg.offset, seg.vaddr, seg.filesz, seg.memsz
            );
        }
        println!();
    }

    if !result.sections.is_empty() {
        println!("# Sections");
        println!("name,addr,size,offset");
        for sec in &result.sections {
            println!(
                "{},0x{:x},{},0x{:x}",
                sec.name, sec.addr, sec.size, sec.offset
            );
        }
    }

    Ok(())
}
