use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead as _, BufReader, BufWriter, Write, stdout},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use clap_stdin::FileOrStdin;
use miette::IntoDiagnostic;

use poro_hash::{BinHash, FromStrRadix as _, Hashtable, WadHash, fst};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    #[command(subcommand)]
    Brex(BrexCommand),
    #[command(subcommand)]
    Hash(HashCommand),
}

#[derive(Subcommand, Clone)]
pub enum HashCommand {
    Compress {
        kind: HashKind,
        input: PathBuf,
        output: PathBuf,
    },
    Decompress {
        kind: HashKind,
        input: PathBuf,
        output: PathBuf,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, clap::ValueEnum)]
pub enum HashKind {
    Bin,
    Wad,
}

#[derive(Subcommand, Clone)]
pub enum BrexCommand {
    Encode {
        input: FileOrStdin,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Decode {
        input: FileOrStdin,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn output_or_stdout(path: Option<PathBuf>) -> Result<Box<dyn std::io::Write>, std::io::Error> {
    path.map(|path| {
        File::create(path)
            .map(BufWriter::new)
            .map(Box::new)
            .map(|w| w as Box<dyn Write>)
    })
    .unwrap_or_else(|| Ok(Box::new(stdout())))
}

fn main() -> miette::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Brex(command) => match command {
            BrexCommand::Encode { input, output } => {
                let mut output = output_or_stdout(output).into_diagnostic()?;
                let mut lines = BufReader::new(input.into_reader().into_diagnostic()?).lines();
                while let Some(Ok(line)) = lines.next() {
                    eprintln!("{line}");
                    let encoded = brex::encode(line.trim()).into_diagnostic()?;
                    writeln!(output, "{encoded}").into_diagnostic()?;
                }
            }
            BrexCommand::Decode { input, output } => {
                let mut output = output_or_stdout(output).into_diagnostic()?;
                let mut lines = BufReader::new(input.into_reader().into_diagnostic()?).lines();
                while let Some(Ok(line)) = lines.next() {
                    eprintln!("{line}");
                    let encoded = brex::decode(line.trim()).into_diagnostic()?;
                    writeln!(output, "{encoded}").into_diagnostic()?;
                }
            }
        },
        Commands::Hash(command) => match command {
            HashCommand::Compress {
                kind,
                input,
                output,
            } => {
                let file = BufReader::new(File::open(&input).into_diagnostic()?);

                println!("Reading {input:?}...");
                let trie: fst::Set<Vec<u8>> = match kind {
                    HashKind::Bin => {
                        let mut entries = BTreeMap::new();
                        for line in file.lines() {
                            let line = line.into_diagnostic()?;
                            let (hash, value) = line.split_once(' ').unwrap();
                            let hash = BinHash::from_str_radix(hash, 16).into_diagnostic()?;
                            entries.insert(hash, value.trim().to_string());
                        }
                        let table = Hashtable::from(entries);
                        println!("Compressing {} entries...", table.hashes.len());
                        table.into()
                    }
                    HashKind::Wad => {
                        let mut entries = BTreeMap::new();
                        for line in file.lines() {
                            let line = line.into_diagnostic()?;
                            let (hash, value) = line.split_once(' ').unwrap();
                            let hash = WadHash::from_str_radix(hash, 16).into_diagnostic()?;
                            entries.insert(hash, value.trim().to_string());
                        }
                        let table = Hashtable::from(entries);
                        println!("Compressing {} entries...", table.hashes.len());
                        table.into()
                    }
                };

                if let Some(parent) = output.parent() {
                    std::fs::create_dir_all(parent).into_diagnostic()?;
                }
                let mut output = BufWriter::new(File::create(&output).into_diagnostic()?);
                println!("Writing to {output:?}...");

                output
                    .write_all(trie.into_fst().as_bytes())
                    .into_diagnostic()?;
            }
            HashCommand::Decompress {
                kind,
                input,
                output,
            } => {
                println!("Decompressing {input:?}...");
                let set = fst::Set::new(std::fs::read(&input).unwrap()).unwrap();
                match kind {
                    HashKind::Wad => {
                        let table = Hashtable::<WadHash>::from_fst(set).into_diagnostic()?;

                        println!("table w/ {} entries decompressed.", table.hashes.len());

                        let mut output =
                            BufWriter::new(std::fs::File::create(&output).into_diagnostic()?);
                        let mut entries = table.hashes.into_iter().collect::<Vec<_>>();
                        entries.sort_unstable_by(|a, b| a.1.cmp(&b.1));
                        for (hash, value) in entries {
                            writeln!(output, "{hash} {value}").into_diagnostic()?;
                        }
                    }
                    HashKind::Bin => {
                        let table = Hashtable::<BinHash>::from_fst(set).into_diagnostic()?;

                        println!("table w/ {} entries decompressed.", table.hashes.len());

                        let mut output =
                            BufWriter::new(std::fs::File::create(&output).into_diagnostic()?);
                        let mut entries = table.hashes.into_iter().collect::<Vec<_>>();
                        entries.sort_unstable_by(|a, b| a.1.cmp(&b.1));
                        for (hash, value) in entries {
                            writeln!(output, "{hash} {value}").into_diagnostic()?;
                        }
                    }
                }
            }
        },
    }
    Ok(())
}
