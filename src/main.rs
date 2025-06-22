mod cli;
mod privacy;
mod processor;
mod analyzer;
mod remover;
mod utils;

use std::path::Path;
use walkdir::WalkDir;
use cli::Config;
use processor::ImageProcessor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_args()?;
    
    // Validate input directory
    if !Path::new(&config.input_dir).is_dir() {
        eprintln!("Error: Input path '{}' is not a directory", config.input_dir);
        std::process::exit(1);
    }

    // Create output directory if specified
    if let Some(ref out_dir) = config.output_dir {
        std::fs::create_dir_all(out_dir)?;
    }

    if config.dry_run {
        println!("DRY RUN MODE - No files will be modified");
    }

    println!("Privacy level: {:?}", config.privacy_level);
    config.print_privacy_explanation();

    let processor = ImageProcessor::new(config);
    let stats = run_processing(&processor)?;

    print_summary(&stats);
    Ok(())
}

fn run_processing(processor: &ImageProcessor) -> Result<ProcessingStats, Box<dyn std::error::Error>> {
    let mut stats = ProcessingStats::new();

    let walker = if processor.config().recursive {
        WalkDir::new(&processor.config().input_dir)
    } else {
        WalkDir::new(&processor.config().input_dir).max_depth(1)
    };

    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error walking directory: {}", e);
                stats.errors += 1;
                continue;
            }
        };

        if entry.file_type().is_file() {
            let path = entry.path();
            
            if utils::is_supported_image(path) {
                match processor.process_image(path) {
                    Ok(had_privacy_data) => {
                        if processor.config().verbose || processor.config().dry_run {
                            println!("Processed: {}", path.display());
                        }
                        stats.processed += 1;
                        if had_privacy_data {
                            stats.privacy_data_found += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error processing {}: {}", path.display(), e);
                        stats.errors += 1;
                    }
                }
            }
        }
    }

    Ok(stats)
}

fn print_summary(stats: &ProcessingStats) {
    println!("\nSummary:");
    println!("Files processed: {}", stats.processed);
    println!("Files with privacy data found: {}", stats.privacy_data_found);
    println!("Errors: {}", stats.errors);
}

#[derive(Default)]
struct ProcessingStats {
    processed: u32,
    privacy_data_found: u32,
    errors: u32,
}

impl ProcessingStats {
    fn new() -> Self {
        Self::default()
    }
}