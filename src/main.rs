use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    dir: PathBuf,
    #[arg(short, long)]
    depth: Option<u32>,
    #[arg(short = 'a', long)]
    show_hidden: bool,
    #[arg(short = 's', long)]
    show_summary: bool,
    #[arg(short, long)]
    follow_symlinks: bool,
}

#[derive(Debug, Default)]
struct Summary {
    dir_count: u32,
    file_count: u32,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    fn visit_dirs(
        dir: &Path,
        prefix: &mut Vec<&str>,
        depth: Option<u32>,
        cli: &Cli,
    ) -> std::io::Result<Summary> {
        let mut summary = Summary::default();
        if dir.is_dir() && depth.is_none_or(|d| d > 0) {
            let mut entries = fs::read_dir(dir)?
                .filter_map(|e| e.inspect_err(|e| eprintln!("{}", e)).ok())
                // Hidden files
                .filter(|e| cli.show_hidden || !e.file_name().as_encoded_bytes().starts_with(b"."))
                .peekable();
            while let Some(entry) = entries.next() {
                let path = entry.path();
                let file_type = entry.file_type()?;
                let raw_file_name = entry.file_name();

                // Printing
                let (arm, padding) = if entries.peek().is_some() {
                    ("+-- ", "|   ")
                } else {
                    ("`-- ", "    ")
                };
                println!("{}{}{}", prefix.concat(), arm, raw_file_name.display());

                // Post-printing processing
                if !cli.follow_symlinks && file_type.is_symlink() {
                    continue;
                }

                // Recursion
                prefix.push(padding);
                let Summary {
                    dir_count,
                    file_count,
                } = visit_dirs(
                    &path,
                    prefix,
                    depth.map(|d| d - 1),
                    cli,
                )?;
                prefix.pop();

                // Summary
                summary.dir_count += dir_count;
                summary.file_count += file_count;
                if file_type.is_dir() {
                    summary.dir_count += 1;
                } else if file_type.is_file() {
                    summary.file_count += 1;
                }
            }
        }
        Ok(summary)
    }

    // Print the root
    println!("{}", cli.dir.display());

    // Enter the recursion
    let Summary {
        dir_count,
        file_count,
    } = visit_dirs(&cli.dir, &mut vec![], cli.depth, &cli)?;

    // Print summary
    if cli.show_summary {
        println!(
            "\n{} {}, {} {}",
            dir_count,
            if dir_count == 1 {
                "directory"
            } else {
                "directories"
            },
            file_count,
            if file_count == 1 { "file" } else { "files" }
        );
    }

    Ok(())
}
