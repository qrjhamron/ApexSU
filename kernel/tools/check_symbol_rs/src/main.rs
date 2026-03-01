// check_symbol: Host build tool that verifies all undefined symbols in a kernel
// module (.ko) are present and globally visible in the vmlinux ELF binary.
// Usage: check_symbol <ko_elf> <vmlinux>
// Exit code 0 if all symbols resolve, 1 if any are missing or on errors.

use anyhow::{bail, Context, Result};
use object::elf;
use object::read::elf::{ElfFile64, FileHeader, Sym};
use object::Endianness;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::process;

/// Symbol resolution status found in vmlinux.
enum VmlinuxSymbol {
    /// Symbol is defined with the given ST_BIND value.
    Defined { binding: u8 },
    /// Symbol entry exists but is itself undefined.
    Undefined,
}

/// Build a lookup table of symbol name -> resolution status from the vmlinux ELF.
fn build_vmlinux_map<'data>(
    vmlinux: &ElfFile64<'data, Endianness>,
) -> Result<HashMap<&'data [u8], VmlinuxSymbol>> {
    let endian = vmlinux.endian();
    let data = vmlinux.data();
    let header = vmlinux.elf_header();
    let sections = header.sections(endian, data)?;
    let symbols = sections.symbols(endian, data, elf::SHT_SYMTAB)?;

    let mut map = HashMap::new();

    for index in 1..symbols.len() {
        let symbol = symbols.symbol(object::SymbolIndex(index))?;
        let name = symbol.name(endian, symbols.strings())?;
        if name.is_empty() {
            continue;
        }
        let entry = if symbol.st_shndx(endian) == elf::SHN_UNDEF {
            VmlinuxSymbol::Undefined
        } else {
            VmlinuxSymbol::Defined {
                binding: symbol.st_bind(),
            }
        };
        map.insert(name, entry);
    }

    Ok(map)
}

fn run() -> Result<bool> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        bail!(
            "Usage: {} <ko_elf> <vmlinux>",
            args.first().map_or("check_symbol", |s| s.as_str())
        );
    }

    let ko_path = &args[1];
    let vmlinux_path = &args[2];

    let ko_data = fs::read(ko_path).with_context(|| format!("Cannot open file {ko_path}"))?;
    let vmlinux_data =
        fs::read(vmlinux_path).with_context(|| format!("Cannot open file {vmlinux_path}"))?;

    let ko_elf = ElfFile64::<Endianness>::parse(ko_data.as_slice())
        .with_context(|| format!("{ko_path} is not a valid ELF file"))?;
    let vmlinux_elf = ElfFile64::<Endianness>::parse(vmlinux_data.as_slice())
        .with_context(|| format!("{vmlinux_path} is not a valid ELF file"))?;

    let ko_endian = ko_elf.endian();
    let ko_header = ko_elf.elf_header();
    let ko_sections = ko_header.sections(ko_endian, ko_elf.data())?;
    let ko_symbols = ko_sections.symbols(ko_endian, ko_elf.data(), elf::SHT_SYMTAB)?;

    if ko_symbols.is_empty() {
        bail!("No symbol table found in {ko_path}");
    }

    let vmlinux_map = build_vmlinux_map(&vmlinux_elf)
        .with_context(|| format!("Failed to read symbols from {vmlinux_path}"))?;

    if vmlinux_map.is_empty() {
        bail!("No symbol table found in {vmlinux_path}");
    }

    let mut has_error = false;

    for index in 1..ko_symbols.len() {
        let symbol = ko_symbols.symbol(object::SymbolIndex(index))?;

        if symbol.st_shndx(ko_endian) != elf::SHN_UNDEF {
            continue;
        }

        let name = symbol.name(ko_endian, ko_symbols.strings())?;
        if name.is_empty() {
            continue;
        }

        let name_str = std::str::from_utf8(name).unwrap_or("<invalid utf-8>");

        match vmlinux_map.get(name) {
            Some(VmlinuxSymbol::Defined { binding }) => {
                if *binding != elf::STB_GLOBAL && *binding != elf::STB_WEAK {
                    eprintln!(
                        "Warning: Symbol '{name_str}' is defined in {vmlinux_path} but not global (binding={binding})"
                    );
                }
            }
            Some(VmlinuxSymbol::Undefined) | None => {
                eprintln!("Error: Symbol '{name_str}' not found or undefined in {vmlinux_path}");
                has_error = true;
            }
        }
    }

    Ok(has_error)
}

fn main() {
    match run() {
        Ok(has_error) => {
            if has_error {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {e:#}");
            process::exit(1);
        }
    }
}
