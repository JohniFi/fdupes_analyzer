use humanize_bytes::humanize_bytes_binary;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Debug)]
struct FileBlock {
    bytes: usize,
    paths: Vec<String>,
}

fn main() -> io::Result<()> {
    // Get the command-line arguments
    let args: Vec<String> = env::args().collect();

    // Ensure that a file path is provided
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    // The file path is the first argument after the program name
    let file_path = &args[1];

    // Open the file
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut blocks = Vec::new();
    let mut current_block: Option<FileBlock> = None;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        // Check for empty line indicating the end of a block
        if line.is_empty() {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }
        } else if line.ends_with("bytes each:") {
            // Parse the first line to extract the byte count
            if let Some(bytes_str) = line.split_whitespace().next() {
                if let Ok(bytes) = bytes_str.parse::<usize>() {
                    current_block = Some(FileBlock {
                        bytes,
                        paths: Vec::new(),
                    });
                }
            }
        } else if let Some(ref mut block) = current_block {
            // Add file paths to the current block
            block.paths.push(line.to_string());
        }
    }

    // Push the last block if any
    if let Some(block) = current_block {
        blocks.push(block);
    }

    // Sort the blocks by the number of bytes
    blocks.sort_by(|a, b| a.bytes.cmp(&b.bytes));

    let mut sum: usize = 0;
    let mut biggest_group: usize = 0;

    // Print the collected and sorted blocks
    for block in &blocks {
        if block.bytes < 1024 * 1024 {
            // ignore files smaller than 1 MiB
            continue;
        }

        let group_size = block.bytes * (block.paths.len() - 1);
        if group_size > biggest_group {
            biggest_group = group_size;
        }
        sum += group_size;

        println!(
            "{} bytes each, {} total duplicates:",
            humanize_bytes_binary!(block.bytes),
            humanize_bytes_binary!(group_size)
        );
        for path in &block.paths {
            println!("{}", path);
        }
        println!(); // For visual separation between blocks
    }

    println!("sum of duplicates: {}", humanize_bytes_binary!(sum));
    println!(
        "smallest file: {}",
        humanize_bytes_binary!(blocks.first().unwrap().bytes)
    );
    println!(
        "biggest file: {}",
        humanize_bytes_binary!(blocks.last().unwrap().bytes)
    );
    println!("biggest group: {}", humanize_bytes_binary!(biggest_group));

    Ok(())
}
