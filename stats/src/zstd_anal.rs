use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::File,
    io::{BufRead, BufReader},
};

use human_bytes::human_bytes;
use rand::rand_core::block;

fn common_prefix(a: &str, b: &str) -> usize {
    a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count()
}

fn zstd_ratio(data: &[u8]) -> (usize, usize) {
    use zstd::stream::encode_all;
    let compressed = encode_all(data, 3).unwrap();
    (compressed.len(), data.len())
}

fn main() {
    let path = env::args().nth(1).unwrap();
    let file = BufReader::new(File::open(path).unwrap());

    let mut entries = Vec::<String>::new();

    for line in file.lines() {
        let line = line.unwrap();
        let (_, path) = line.split_once(' ').unwrap();
        entries.push(path.to_string());
    }

    let total = entries.len();

    println!("entries: {total}");

    let joined = entries.join("\n");

    let block_sizes = [32, 64, 128, 512, 1024, 2048, 4096, 8192];

    let (comp, orig) = zstd_ratio(joined.as_bytes());
    println!(
        "full zstd: {} -> {} ({:.4}%)",
        human_bytes::human_bytes(orig as f64),
        human_bytes::human_bytes(comp as f64),
        (comp as f64 / orig as f64) * 100.
    );
    for block_size in block_sizes {
        let mut raw = (usize::MAX, usize::MIN);
        let mut comp = (usize::MAX, usize::MIN);
        let mut sum = (0, 0);
        let mut count = 0;
        for window in entries.chunks(block_size) {
            let joined = window.join("\n");
            let (b_comp, b_raw) = zstd_ratio(joined.as_bytes());
            raw.0 = raw.0.min(b_raw);
            raw.1 = raw.1.max(b_raw);
            comp.0 = comp.0.min(b_comp);
            comp.1 = comp.1.max(b_comp);

            sum.0 += b_raw;
            sum.1 += b_comp;

            count += 1;
        }
        let raw = (raw.0 as f64, raw.1 as f64);
        let comp = (comp.0 as f64, comp.1 as f64);
        let sum = (sum.0 as f64 /* raw */, sum.1 as f64 /* comp */);
        let count = count as f64;

        println!("[{block_size}] (compressed/raw):");
        println!(
            " - total: {} / {} ({:.4}%)",
            human_bytes(sum.1),
            human_bytes(sum.0),
            (sum.1 / sum.0) * 100.,
        );
        println!(
            " -  min: {} / {} ({:.4}%)",
            human_bytes(comp.0),
            human_bytes(raw.0),
            (comp.0 / raw.0) * 100.
        );
        let mean = (sum.1 / count, sum.0 / count);
        println!(
            " - mean: {} / {} ({:.4}%)",
            human_bytes(mean.0),
            human_bytes(mean.1),
            (mean.0 / mean.1) * 100.
        );
        println!(
            " -  max: {} / {} ({:.4}%)",
            human_bytes(comp.1),
            human_bytes(raw.1),
            (comp.1 / raw.1) * 100.
        );
    }
}
