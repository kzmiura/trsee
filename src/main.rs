use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    dir: PathBuf,
    #[arg(short, long)]
    depth: Option<u32>,
}

fn main() -> std::io::Result<()> {
    fn visit_dirs(dir: &Path, prefix: impl AsRef<str>, depth: Option<u32>) -> std::io::Result<()> {
        if dir.is_dir() && depth.is_none_or(|d| d > 0) {
            let prefix = prefix.as_ref();
            let mut entries = fs::read_dir(dir)?.into_iter().peekable();
            while let Some(entry) = entries.next() {
                let entry = entry?;
                let path = entry.path();
                let is_last = entries.peek().is_none();

                if let Some(file_name) = path.file_name() {
                    println!(
                        "{}{}{}",
                        prefix,
                        if is_last {
                            "\u{2514}\u{2500}\u{2500} "
                        } else {
                            "\u{251c}\u{2500}\u{2500} "
                        },
                        file_name.to_string_lossy()
                    );
                }

                visit_dirs(
                    &path,
                    format!("{}{}", prefix, if is_last { "    " } else { "\u{2502}   " }),
                    depth.map(|d| d - 1),
                )?;
            }
        }

        Ok(())
    }

    let cli = Cli::parse();

    println!("{}", cli.dir.display());
    visit_dirs(&cli.dir, "", cli.depth)?;

    Ok(())
}
