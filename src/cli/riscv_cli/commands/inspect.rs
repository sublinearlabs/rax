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
pub fn execute_inspect(binary: &str, format: &str, output: Option<&str>) -> CliResult<()> {
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

    // Format output as string
    let output_text = match format {
        "text" => format_inspect_text(&result)?,
        "json" => format_inspect_json(&result)?,
        "csv" => format_inspect_csv(&result)?,
        _ => {
            return Err(CliError::new(format!(
                "Unknown output format: '{}'. Use: text, json, csv",
                format
            )));
        }
    };

    // Display to stdout
    println!("{}", output_text);

    // Save to file if requested
    if let Some(file_path) = output {
        write_inspect_to_file(file_path, &output_text)?;
        print_info(&format!("Inspect output written to: {}", file_path));
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
fn format_inspect_text(result: &InspectResult) -> CliResult<String> {
    let mut output = String::new();

    output.push_str(&format!("\n{}\n", "ELF Header Information".bold()));
    output.push_str(&format!("{}\n", "-".repeat(80)));
    output.push_str(&format!("  Architecture:       {}\n", result.architecture));
    output.push_str(&format!(
        "  File Size:          {} bytes\n",
        result.file_size
    ));
    output.push_str(&format!(
        "  Entry Point:        0x{:x}\n",
        result.entry_point
    ));
    output.push_str(&format!("  Sections:           {}\n", result.num_sections));
    output.push_str(&format!("  Segments:           {}\n", result.num_segments));
    output.push_str("\n");

    if !result.segments.is_empty() {
        output.push_str(&format!("{}\n", "Program Headers (Segments)".bold()));
        output.push_str(&format!("{}\n", "-".repeat(80)));
        output.push_str(&format!(
            "{:<16} {:<12} {:<16} {:<10} {:<10}\n",
            "Type", "Offset", "VAddr", "FileSize", "MemSize"
        ));
        output.push_str(&format!("{}\n", "-".repeat(80)));
        for seg in &result.segments {
            output.push_str(&format!(
                "{:<16} 0x{:<10x} 0x{:<14x} {:<10} {:<10}\n",
                seg.ptype, seg.offset, seg.vaddr, seg.filesz, seg.memsz
            ));
        }
        output.push_str("\n");
    }

    if !result.sections.is_empty() {
        output.push_str(&format!("{}\n", "Sections".bold()));
        output.push_str(&format!("{}\n", "-".repeat(80)));
        output.push_str(&format!(
            "{:<32} {:<16} {:<10} {:<12}\n",
            "Name", "Address", "Size", "Offset"
        ));
        output.push_str(&format!("{}\n", "-".repeat(80)));
        for sec in &result.sections {
            output.push_str(&format!(
                "{:<32} 0x{:<14x} {:<10} 0x{:<10x}\n",
                sec.name, sec.addr, sec.size, sec.offset
            ));
        }
    }

    output.push_str("\n");
    Ok(output)
}

/// Format inspect output as JSON
fn format_inspect_json(result: &InspectResult) -> CliResult<String> {
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

    Ok(serde_json::to_string_pretty(&json).unwrap())
}

/// Format inspect output as CSV
fn format_inspect_csv(result: &InspectResult) -> CliResult<String> {
    let mut output = String::new();

    output.push_str("# ELF Header\n");
    output.push_str(&format!("architecture,{}\n", result.architecture));
    output.push_str(&format!("file_size,{}\n", result.file_size));
    output.push_str(&format!("entry_point,0x{:x}\n", result.entry_point));
    output.push_str(&format!("num_sections,{}\n", result.num_sections));
    output.push_str(&format!("num_segments,{}\n", result.num_segments));
    output.push_str("\n");

    if !result.segments.is_empty() {
        output.push_str("# Program Headers\n");
        output.push_str("type,offset,vaddr,filesz,memsz\n");
        for seg in &result.segments {
            output.push_str(&format!(
                "{},0x{:x},0x{:x},{},{}\n",
                seg.ptype, seg.offset, seg.vaddr, seg.filesz, seg.memsz
            ));
        }
        output.push_str("\n");
    }

    if !result.sections.is_empty() {
        output.push_str("# Sections\n");
        output.push_str("name,addr,size,offset\n");
        for sec in &result.sections {
            output.push_str(&format!(
                "{},0x{:x},{},0x{:x}\n",
                sec.name, sec.addr, sec.size, sec.offset
            ));
        }
    }

    Ok(output)
}

/// Write inspect output to a file
fn write_inspect_to_file(file_path: &str, content: &str) -> CliResult<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(file_path).map_err(|e| {
        CliError::new(format!(
            "Failed to create output file '{}': {}",
            file_path, e
        ))
    })?;

    file.write_all(content.as_bytes())
        .map_err(|e| CliError::new(format!("Failed to write to file '{}': {}", file_path, e)))?;

    Ok(())
}
