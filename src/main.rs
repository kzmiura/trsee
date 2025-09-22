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
        prefix: impl AsRef<str>,
        depth: Option<u32>,
        cli: &Cli,
    ) -> std::io::Result<Summary> {
        let mut summary = Summary::default();
        if depth.is_none_or(|d| d > 0) {
            if dir.is_dir() {
                let mut entries = fs::read_dir(dir)?.into_iter().peekable();
                while let Some(entry) = entries.next() {
                    let entry = entry?;
                    let path = entry.path();
                    let raw_file_name = entry.file_name();
                    let file_name = raw_file_name.to_string_lossy();

                    if !cli.show_hidden && file_name.starts_with('.') {
                        continue;
                    }

                    let (arm, padding) = if entries.peek().is_some() {
                        ("+-- ", "|   ")
                    } else {
                        ("`-- ", "    ")
                    };

                    let prefix = prefix.as_ref();
                    println!("{}{}{}", prefix, arm, file_name);

                    if !cli.follow_symlinks && entry.file_type()?.is_symlink() {
                        continue;
                    }

                    let Summary {
                        dir_count,
                        file_count,
                    } = visit_dirs(
                        &path,
                        prefix.to_owned() + padding,
                        depth.map(|d| d - 1),
                        cli,
                    )?;

                    if path.is_dir() {
                        summary.dir_count += dir_count + 1;
                    } else {
                        summary.file_count += file_count + 1;
                    }
                }
            } else {
                summary.file_count += 1;
            }
        }

        Ok(summary)
    }

    println!("{}", cli.dir.display());
    let Summary {
        dir_count,
        file_count,
    } = visit_dirs(&cli.dir, "", cli.depth, &cli)?;
    if cli.show_summary {
        println!(
            "\n{} {}, {} {}",
            dir_count,
            if dir_count > 1 {
                "directories"
            } else {
                "directory"
            },
            file_count,
            if file_count > 1 { "files" } else { "file" }
        );
    }

    Ok(())
}
