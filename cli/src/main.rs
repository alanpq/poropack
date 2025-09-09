use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead as _, BufReader, BufWriter, Write},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use miette::IntoDiagnostic;

use hashtools::{Hasher, Hashtable, WadHasher, fst};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    Compress { input: PathBuf, output: PathBuf },
    Decompress { input: PathBuf, output: PathBuf },
}

fn main() -> miette::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Compress { input, output } => {
            let file = BufReader::new(File::open(&input).into_diagnostic()?);

            println!("Reading {input:?}...");
            let mut entries = BTreeMap::new();
            for line in file.lines() {
                let line = line.into_diagnostic()?;
                let (hash, value) = line.split_once(' ').unwrap();
                let hash = u64::from_str_radix(hash, 16).into_diagnostic()?;
                entries.insert(hash, value.trim().to_string());
            }

            let table = Hashtable::<WadHasher>::from(entries);

            println!("Compressing {} entries...", table.hashes.len());
            let trie: fst::Set<Vec<u8>> = table.into();
            let mut output = BufWriter::new(File::create(&output).into_diagnostic()?);
            println!("Writing to {output:?}...");

            output
                .write_all(trie.into_fst().as_bytes())
                .into_diagnostic()?;
        }
        Commands::Decompress { input, output } => {
            println!("Decompressing {input:?}...");
            let set = fst::Set::new(std::fs::read(&input).unwrap()).unwrap();
            let table = Hashtable::<WadHasher>::try_from(set).into_diagnostic()?;

            println!("table w/ {} entries decompressed.", table.hashes.len());

            let mut output = BufWriter::new(std::fs::File::create(&output).into_diagnostic()?);
            let mut entries = table.hashes.into_iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|a, b| a.1.cmp(&b.1));
            for (hash, value) in entries {
                writeln!(output, "{hash:0>16x} {value}").into_diagnostic()?;
            }
        }
    }
    Ok(())
}
