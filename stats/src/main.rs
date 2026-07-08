use anyhow::Context as _;
use clap::{Parser, Subcommand};
use dhat::HeapStats;
use human_bytes::human_bytes;
use poro_hash::{
    ltk_hash::WadHash,
    poroblob::{self, LruBlocks},
};
use rand::seq::SliceRandom;
use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::File,
    io::{BufRead, BufReader, BufWriter},
    num::NonZero,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Import {
        input: PathBuf,
        output: PathBuf,
    },
    Load {
        input: PathBuf,
        #[arg(long)]
        lru: usize,
    },
}

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    let args = Args::parse();

    match args.command {
        Command::Import { input, output } => {
            let input = File::open(input)
                .map(BufReader::new)
                .context("opening input file for read")?;

            let mut output = File::create(output)
                .map(BufWriter::new)
                .context("opening output file for write")?;

            let mut builder = poroblob::Builder::<WadHash>::new(2048);
            builder.extend(input.lines().filter_map(|line| {
                let line = line.ok()?;
                Some(line.split_once(' ')?.1.to_string())
            }))?;
            builder.write(&mut output)?;
        }
        Command::Load { input, lru } => {
            let mut input = File::open(input)
                .map(BufReader::new)
                .context("opening input file for read")?;

            let blob = poroblob::Poroblob::<WadHash, _>::from_reader(&mut input)?;

            let input = input.into_inner();
            let mmap = unsafe { memmap2::Mmap::map(&input)? };

            let blob = blob.with_block_data(LruBlocks::new(mmap, NonZero::new(lru).unwrap()));

            let min = blob.blocks.iter().map(|b| b.len).min().unwrap_or_default() as f64;
            let max = blob.blocks.iter().map(|b| b.len).max().unwrap_or_default() as f64;
            let avg =
                blob.blocks.iter().map(|b| b.len).sum::<u32>() as f64 / blob.blocks.len() as f64;
            println!(
                "block size (min/avg/max): {}/{}/{}",
                human_bytes(min),
                human_bytes(avg),
                human_bytes(max)
            );

            let mut rng = rand::rng();

            println!("{} blocks", blob.blocks.len());
            println!("{} entries", blob.index.len());
            let mut keys = blob.index.keys().copied().collect::<Vec<_>>();
            keys.shuffle(&mut rng);

            // get memory usage

            let then = Instant::now();
            let count = 10_000;

            let usage = HeapStats::get().curr_bytes as f64;
            for key in keys.into_iter().take(count) {
                blob.get(key).expect("key lookup failed");
            }
            let usage = (HeapStats::get().curr_bytes as f64 - usage);

            let elapsed = then.elapsed();
            let rate = count as f64 / elapsed.as_secs_f64();

            println!("{rate:#.4} lookups/sec");
            println!("{} allocated during lookup loop", human_bytes(usage));

            // get memory usage again -> print difference

            // let mut s = String::new();
            // std::io::stdin().read_line(&mut s).unwrap();
        }
    }

    Ok(())
}
