use clap::{Arg, Command, ValueEnum};
use crate::privacy::PrivacyLevel;

#[derive(Debug, Clone)]
pub struct Config {
    pub input_dir: String,
    pub output_dir: Option<String>,
    pub recursive: bool,
    pub create_backup: bool,
    pub privacy_level: PrivacyLevel,
    pub verbose: bool,
    pub dry_run: bool,
}

impl Config {
    pub fn from_args() -> Result<Self, Box<dyn std::error::Error>> {
        let matches = Command::new("privacy-exif-cleaner")
            .version("1.0")
            .about("Removes privacy-sensitive information from EXIF data while preserving technical metadata")
            .arg(
                Arg::new("input")
                    .short('i')
                    .long("input")
                    .value_name("DIR")
                    .help("Input directory containing images")
                    .required(true),
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("DIR")
                    .help("Output directory (optional - will modify in-place if not specified)"),
            )
            .arg(
                Arg::new("recursive")
                    .short('r')
                    .long("recursive")
                    .help("Process subdirectories recursively")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("backup")
                    .short('b')
                    .long("backup")
                    .help("Create backup files with .bak extension")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("privacy_level")
                    .short('p')
                    .long("privacy")
                    .value_enum::<PrivacyLevel>()
                    .default_value("standard")
                    .help("Privacy level: minimal, standard, strict, or paranoid"),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Show detailed information about data being removed")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("dry_run")
                    .short('n')
                    .long("dry-run")
                    .help("Show what would be removed without making changes")
                    .action(clap::ArgAction::SetTrue),
            )
            .get_matches();

        Ok(Config {
            input_dir: matches.get_one::<String>("input").unwrap().clone(),
            output_dir: matches.get_one::<String>("output").cloned(),
            recursive: matches.get_flag("recursive"),
            create_backup: matches.get_flag("backup"),
            privacy_level: matches.get_one::<PrivacyLevel>("privacy_level").unwrap().clone(),
            verbose: matches.get_flag("verbose"),
            dry_run: matches.get_flag("dry_run"),
        })
    }

    pub fn print_privacy_explanation(&self) {
        println!("\nPrivacy settings for {:?} level:", self.privacy_level);
        match self.privacy_level {
            PrivacyLevel::Minimal => {
                println!("• Removes: GPS coordinates, location data");
                println!("• Preserves: All camera settings, timestamps, device info");
            }
            PrivacyLevel::Standard => {
                println!("• Removes: GPS data, camera serial numbers, unique device IDs");
                println!("• Preserves: Camera model, settings, timestamps, non-identifying technical data");
            }
            PrivacyLevel::Strict => {
                println!("• Removes: GPS, device IDs, timestamps, user comments, software info");
                println!("• Preserves: Camera settings (ISO, aperture, etc.), color profiles");
            }
            PrivacyLevel::Paranoid => {
                println!("• Removes: All metadata except essential technical camera settings");
                println!("• Preserves: Only ISO, aperture, focal length, exposure time");
            }
        }
        println!();
    }
}