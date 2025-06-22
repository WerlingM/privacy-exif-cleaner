use std::path::{Path, PathBuf};
use std::fs;
use crate::cli::Config;
use crate::analyzer::ExifAnalyzer;
use crate::remover::MetadataRemover;

pub struct ImageProcessor {
    config: Config,
    analyzer: ExifAnalyzer,
    remover: MetadataRemover,
}

impl ImageProcessor {
    pub fn new(config: Config) -> Self {
        Self {
            analyzer: ExifAnalyzer::new(),
            remover: MetadataRemover::new(),
            config,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Process a single image file
    pub fn process_image(&self, input_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        // Read the file data
        let file_data = fs::read(input_path)?;
        
        // Analyze what privacy data exists
        let privacy_data = self.analyzer.analyze_privacy_data(
            &file_data, 
            input_path, 
            &self.config.privacy_level, 
            self.config.verbose
        )?;
        
        if privacy_data.is_empty() {
            if self.config.verbose {
                println!("  No privacy-sensitive data found in {}", input_path.display());
            }
            return Ok(false);
        }

        if self.config.dry_run {
            println!("  Would remove {} privacy-sensitive fields from {}", 
                privacy_data.len(), input_path.display());
            return Ok(true);
        }

        // Determine output path
        let output_path = self.get_output_path(input_path)?;

        // Create backup if requested and we're doing in-place modification
        if self.config.create_backup && self.config.output_dir.is_none() {
            self.create_backup(input_path)?;
        }

        // Remove the privacy data
        self.remover.remove_privacy_data(
            input_path,
            &output_path,
            &self.config.privacy_level,
        )?;

        Ok(true)
    }

    /// Determine the output path for a processed file
    fn get_output_path(&self, input_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let output_path = if let Some(ref out_dir) = self.config.output_dir {
            // Save to output directory, preserving filename
            let file_name = input_path.file_name()
                .ok_or("Invalid file name")?;
            PathBuf::from(out_dir).join(file_name)
        } else {
            // In-place modification
            input_path.to_path_buf()
        };

        Ok(output_path)
    }

    /// Create a backup of the original file
    fn create_backup(&self, input_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let backup_path = input_path.with_extension(
            format!("{}.bak", 
                input_path.extension()
                    .unwrap_or_default()
                    .to_string_lossy())
        );
        
        fs::copy(input_path, backup_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::privacy::PrivacyLevel;
    use tempfile::TempDir;

    fn create_test_config() -> Config {
        Config {
            input_dir: "/tmp".to_string(),
            output_dir: None,
            recursive: false,
            create_backup: false,
            privacy_level: PrivacyLevel::Standard,
            verbose: false,
            dry_run: false,
        }
    }

    #[test]
    fn test_output_path_in_place() {
        let config = create_test_config();
        let processor = ImageProcessor::new(config);
        
        let input_path = Path::new("/test/photo.jpg");
        let output_path = processor.get_output_path(input_path).unwrap();
        
        assert_eq!(output_path, input_path);
    }

    #[test]
    fn test_output_path_separate_directory() {
        let mut config = create_test_config();
        config.output_dir = Some("/output".to_string());
        let processor = ImageProcessor::new(config);
        
        let input_path = Path::new("/test/photo.jpg");
        let output_path = processor.get_output_path(input_path).unwrap();
        
        assert_eq!(output_path, Path::new("/output/photo.jpg"));
    }

    #[test]
    fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.jpg");
        fs::write(&test_file, b"fake jpeg data").unwrap();

        let mut config = create_test_config();
        config.create_backup = true;
        let processor = ImageProcessor::new(config);

        processor.create_backup(&test_file).unwrap();

        let backup_file = temp_dir.path().join("test.jpg.bak");
        assert!(backup_file.exists());
        
        let backup_content = fs::read(backup_file).unwrap();
        assert_eq!(backup_content, b"fake jpeg data");
    }
}