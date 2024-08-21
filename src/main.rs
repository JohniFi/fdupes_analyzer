use humanize_bytes::humanize_bytes_binary;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};

// Define a tree structure using a BTreeMap
#[derive(Debug)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
}

impl TreeNode {
    fn new() -> Self {
        TreeNode {
            children: BTreeMap::new(),
        }
    }

    fn add_path(&mut self, components: &[String]) {
        if let Some((first, rest)) = components.split_first() {
            self.children
                .entry(first.clone())
                .or_insert_with(TreeNode::new)
                .add_path(rest);
        }
    }

    fn print(&self, prefix: &str, is_last: bool) {
        let prefix_component = if is_last { "└── " } else { "├── " };
        for (i, (name, child)) in self.children.iter().enumerate() {
            let is_last = i == self.children.len() - 1;
            println!("{}{}{}", prefix, prefix_component, name);
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            child.print(&new_prefix, is_last);
        }
    }
}

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
            block.paths.push(line.to_owned());
        }
    }

    // Push the last block if any
    if let Some(block) = current_block {
        blocks.push(block);
    }

    // Sort the blocks by the number of bytes
    blocks.sort_by(|a, b| (a.bytes * (a.paths.len() - 1)).cmp(&(b.bytes * (b.paths.len() - 1))));

    let mut sum: usize = 0;
    let mut biggest_file_size: usize = 0;
    let mut tree = TreeNode::new();

    // Print the collected and sorted blocks
    for block in &blocks {
        if block.bytes < 10 * 1024 * 1024 {
            // ignore files smaller than 1 MiB
            continue;
        }

        if block.bytes > biggest_file_size {
            biggest_file_size = block.bytes;
        }

        let group_size = block.bytes * (block.paths.len() - 1);
        sum += group_size;

        println!(
            "{} bytes each, {} total duplicates:",
            humanize_bytes_binary!(block.bytes),
            humanize_bytes_binary!(group_size)
        );
        for path in &block.paths {
            let path_components: Vec<String> = path.split('/').map(|s| s.to_string()).collect();
            tree.add_path(&path_components);

            println!("{}", path);
        }
        println!(); // For visual separation between blocks
    }

    // Print the tree structure
    //tree.print("", true);

    let num: usize = blocks.iter().map(|block| block.paths.len() - 1).sum();

    println!(
        "Total number duplicates: {}, using {} of disk space",
        num,
        humanize_bytes_binary!(sum)
    );

    println!(
        "biggest file: {}",
        humanize_bytes_binary!(biggest_file_size)
    );
    println!(
        "biggest group: {}",
        humanize_bytes_binary!(
            blocks.last().unwrap().bytes * (blocks.last().unwrap().paths.len() - 1)
        )
    );

    if let Some(biggest) = blocks.last() {
        for p in biggest.paths.iter() {
            println!("{}", p)
        }
    }
    Ok(())
}
