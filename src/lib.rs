//! Privacy EXIF Cleaner Library
//! 
//! This library provides functionality to remove privacy-sensitive information from image EXIF data
//! while preserving useful technical metadata. It supports different privacy levels and can be used
//! both as a command-line tool and as a library in other Rust projects.

pub mod analyzer;
pub mod cli;
pub mod privacy;
pub mod processor;
pub mod remover;
pub mod utils;

// Re-export main types for easier use
pub use analyzer::{ExifAnalyzer, PrivacyField, PrivacyCategory};
pub use cli::Config;
pub use privacy::{PrivacyLevel, PrivacyPolicy};
pub use processor::ImageProcessor;
pub use remover::MetadataRemover;

/// Main library interface for processing images
pub struct PrivacyExifCleaner {
    processor: ImageProcessor,
}

impl PrivacyExifCleaner {
    /// Create a new instance with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            processor: ImageProcessor::new(config),
        }
    }

    /// Create a new instance with default settings for a given privacy level
    pub fn with_privacy_level(privacy_level: PrivacyLevel) -> Self {
        let config = Config {
            input_dir: ".".to_string(),
            output_dir: None,
            recursive: false,
            create_backup: false,
            privacy_level,
            verbose: false,
            dry_run: false,
        };
        
        Self::new(config)
    }

    /// Process a single image file
    pub fn process_image<P: AsRef<std::path::Path>>(&self, path: P) -> Result<bool, Box<dyn std::error::Error>> {
        self.processor.process_image(path.as_ref())
    }

    /// Analyze what privacy data exists in an image without removing it
    pub fn analyze_image<P: AsRef<std::path::Path>>(&self, path: P) -> Result<Vec<PrivacyField>, Box<dyn std::error::Error>> {
        let file_data = std::fs::read(path.as_ref())?;
        let analyzer = ExifAnalyzer::new();
        analyzer.analyze_privacy_data(&file_data, path.as_ref(), &self.processor.config().privacy_level, false)
    }

    /// Get the current configuration
    pub fn config(&self) -> &Config {
        self.processor.config()
    }
}

/// High-level convenience functions
pub mod convenience {
    use super::*;
    use std::path::Path;

    /// Remove GPS data from a single image file
    pub fn remove_gps_data<P: AsRef<Path>>(image_path: P) -> Result<bool, Box<dyn std::error::Error>> {
        let cleaner = PrivacyExifCleaner::with_privacy_level(PrivacyLevel::Minimal);
        cleaner.process_image(image_path)
    }

    /// Remove standard privacy data (GPS + device IDs + personal info) from a single image
    pub fn remove_standard_privacy_data<P: AsRef<Path>>(image_path: P) -> Result<bool, Box<dyn std::error::Error>> {
        let cleaner = PrivacyExifCleaner::with_privacy_level(PrivacyLevel::Standard);
        cleaner.process_image(image_path)
    }

    /// Remove all metadata except essential camera settings
    pub fn remove_all_except_camera_settings<P: AsRef<Path>>(image_path: P) -> Result<bool, Box<dyn std::error::Error>> {
        let cleaner = PrivacyExifCleaner::with_privacy_level(PrivacyLevel::Paranoid);
        cleaner.process_image(image_path)
    }

    /// Analyze what privacy data exists in an image
    pub fn analyze_privacy_data<P: AsRef<Path>>(image_path: P, privacy_level: PrivacyLevel) -> Result<Vec<PrivacyField>, Box<dyn std::error::Error>> {
        let cleaner = PrivacyExifCleaner::with_privacy_level(privacy_level);
        cleaner.analyze_image(image_path)
    }

    /// Check if an image contains any GPS data
    pub fn has_gps_data<P: AsRef<Path>>(image_path: P) -> Result<bool, Box<dyn std::error::Error>> {
        let privacy_fields = analyze_privacy_data(image_path, PrivacyLevel::Minimal)?;
        Ok(privacy_fields.iter().any(|field| field.category == PrivacyCategory::Location))
    }

    /// Check if an image has any EXIF data at all
    pub fn has_exif_data<P: AsRef<Path>>(image_path: P) -> Result<bool, Box<dyn std::error::Error>> {
        let file_data = std::fs::read(image_path)?;
        let analyzer = ExifAnalyzer::new();
        Ok(analyzer.has_exif_data(&file_data))
    }

    /// Get a summary of privacy categories found in an image
    pub fn get_privacy_summary<P: AsRef<Path>>(image_path: P, privacy_level: PrivacyLevel) -> Result<PrivacySummary, Box<dyn std::error::Error>> {
        let privacy_fields = analyze_privacy_data(image_path, privacy_level)?;
        Ok(PrivacySummary::from_fields(&privacy_fields))
    }
}

/// Summary of privacy data found in an image
#[derive(Debug, Default)]
pub struct PrivacySummary {
    pub has_location_data: bool,
    pub has_device_identifiers: bool,
    pub has_personal_info: bool,
    pub has_timestamps: bool,
    pub has_software_info: bool,
    pub has_metadata: bool,
    pub total_privacy_fields: usize,
}

impl PrivacySummary {
    pub fn from_fields(fields: &[PrivacyField]) -> Self {
        let mut summary = Self::default();
        summary.total_privacy_fields = fields.len();

        for field in fields {
            match field.category {
                PrivacyCategory::Location => summary.has_location_data = true,
                PrivacyCategory::DeviceIdentifier => summary.has_device_identifiers = true,
                PrivacyCategory::PersonalInfo => summary.has_personal_info = true,
                PrivacyCategory::Temporal => summary.has_timestamps = true,
                PrivacyCategory::Software => summary.has_software_info = true,
                PrivacyCategory::Metadata => summary.has_metadata = true,
                PrivacyCategory::Other => {}
            }
        }

        summary
    }

    /// Check if any privacy-sensitive data was found
    pub fn has_privacy_data(&self) -> bool {
        self.total_privacy_fields > 0
    }

    /// Get a human-readable description of privacy issues found
    pub fn describe(&self) -> Vec<String> {
        let mut descriptions = Vec::new();

        if self.has_location_data {
            descriptions.push("Contains GPS location data".to_string());
        }
        if self.has_device_identifiers {
            descriptions.push("Contains device serial numbers or unique identifiers".to_string());
        }
        if self.has_personal_info {
            descriptions.push("Contains personal information (names, copyright, comments)".to_string());
        }
        if self.has_timestamps {
            descriptions.push("Contains timestamp information".to_string());
        }
        if self.has_software_info {
            descriptions.push("Contains software processing information".to_string());
        }
        if self.has_metadata {
            descriptions.push("Contains additional metadata".to_string());
        }

        if descriptions.is_empty() {
            descriptions.push("No privacy-sensitive data found".to_string());
        }

        descriptions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_privacy_exif_cleaner_creation() {
        let config = Config {
            input_dir: ".".to_string(),
            output_dir: None,
            recursive: false,
            create_backup: false,
            privacy_level: PrivacyLevel::Standard,
            verbose: false,
            dry_run: false,
        };

        let cleaner = PrivacyExifCleaner::new(config);
        assert_eq!(cleaner.config().privacy_level, PrivacyLevel::Standard);
    }

    #[test]
    fn test_privacy_exif_cleaner_with_privacy_level() {
        let cleaner = PrivacyExifCleaner::with_privacy_level(PrivacyLevel::Paranoid);
        assert_eq!(cleaner.config().privacy_level, PrivacyLevel::Paranoid);
    }

    #[test]
    fn test_privacy_summary_from_empty_fields() {
        let fields = vec![];
        let summary = PrivacySummary::from_fields(&fields);
        
        assert!(!summary.has_privacy_data());
        assert_eq!(summary.total_privacy_fields, 0);
        assert_eq!(summary.describe(), vec!["No privacy-sensitive data found"]);
    }

    #[test]
    fn test_privacy_summary_with_location_data() {
        use exif::Tag;
        
        let fields = vec![
            PrivacyField {
                tag: Tag::GPSLatitude,
                description: "GPS Latitude: 40.7128".to_string(),
                category: PrivacyCategory::Location,
            }
        ];
        
        let summary = PrivacySummary::from_fields(&fields);
        
        assert!(summary.has_privacy_data());
        assert!(summary.has_location_data);
        assert!(!summary.has_device_identifiers);
        assert_eq!(summary.total_privacy_fields, 1);
        
        let descriptions = summary.describe();
        assert!(descriptions.iter().any(|d| d.contains("GPS location data")));
    }

    #[test]
    fn test_convenience_functions_interface() {
        // These tests just verify the interface compiles and has the right signatures
        // Actual functionality testing would require real image files
        
        use convenience::*;
        
        // Test that functions exist and have correct signatures
        let temp_dir = TempDir::new().unwrap();
        let fake_image = temp_dir.path().join("fake.jpg");
        fs::write(&fake_image, b"fake jpeg data").unwrap();
        
        // These will fail because it's not a real JPEG, but we're testing the interface
        let _ = has_exif_data(&fake_image);
        let _ = analyze_privacy_data(&fake_image, PrivacyLevel::Standard);
        let _ = get_privacy_summary(&fake_image, PrivacyLevel::Minimal);
    }
}