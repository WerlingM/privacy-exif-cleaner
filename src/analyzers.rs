use std::io::Cursor;
use std::path::Path;
use exif::{In, Reader};
use crate::privacy::{PrivacyLevel, PrivacyPolicy};

pub struct ExifAnalyzer {
    reader: Reader,
}

impl ExifAnalyzer {
    pub fn new() -> Self {
        Self {
            reader: Reader::new(),
        }
    }

    /// Analyze what privacy-sensitive data exists in an image
    pub fn analyze_privacy_data(
        &self,
        data: &[u8],
        path: &Path,
        privacy_level: &PrivacyLevel,
        verbose: bool,
    ) -> Result<Vec<PrivacyField>, Box<dyn std::error::Error>> {
        let mut cursor = Cursor::new(data);
        
        let exif = match self.reader.read_from_container(&mut cursor) {
            Ok(exif) => exif,
            Err(_) => return Ok(vec![]), // No EXIF data
        };

        let mut privacy_fields = Vec::new();

        for field in exif.fields() {
            if !PrivacyPolicy::should_preserve_tag(field.tag, privacy_level) {
                let privacy_field = PrivacyField {
                    tag: field.tag,
                    description: format!("{}: {}", 
                        field.tag, 
                        field.display_value().with_unit(&exif)
                    ),
                    category: self.categorize_privacy_field(field.tag),
                };

                privacy_fields.push(privacy_field);
                
                if verbose {
                    println!("  Privacy data found in {}: {} ({})", 
                        path.display(), 
                        privacy_field.description,
                        privacy_field.category
                    );
                }
            }
        }

        Ok(privacy_fields)
    }

    /// Check if an image contains any EXIF data at all
    pub fn has_exif_data(&self, data: &[u8]) -> bool {
        let mut cursor = Cursor::new(data);
        self.reader.read_from_container(&mut cursor).is_ok()
    }

    /// Get all EXIF fields from an image (for debugging/analysis)
    pub fn get_all_exif_fields(&self, data: &[u8]) -> Result<Vec<ExifField>, Box<dyn std::error::Error>> {
        let mut cursor = Cursor::new(data);
        let exif = self.reader.read_from_container(&mut cursor)?;

        let fields = exif.fields()
            .map(|field| ExifField {
                tag: field.tag,
                value: field.display_value().with_unit(&exif).to_string(),
            })
            .collect();

        Ok(fields)
    }

    /// Categorize a privacy field for better user understanding
    fn categorize_privacy_field(&self, tag: exif::Tag) -> PrivacyCategory {
        use exif::Tag;

        match tag {
            Tag::GPSVersionID | Tag::GPSLatitudeRef | Tag::GPSLatitude 
            | Tag::GPSLongitudeRef | Tag::GPSLongitude | Tag::GPSAltitudeRef 
            | Tag::GPSAltitude | Tag::GPSTimeStamp | Tag::GPSSatellites 
            | Tag::GPSStatus | Tag::GPSMeasureMode | Tag::GPSDOP 
            | Tag::GPSSpeedRef | Tag::GPSSpeed | Tag::GPSTrackRef | Tag::GPSTrack 
            | Tag::GPSImgDirectionRef | Tag::GPSImgDirection | Tag::GPSMapDatum 
            | Tag::GPSDestLatitudeRef | Tag::GPSDestLatitude | Tag::GPSDestLongitudeRef 
            | Tag::GPSDestLongitude | Tag::GPSDestBearingRef | Tag::GPSDestBearing 
            | Tag::GPSDestDistanceRef | Tag::GPSDestDistance | Tag::GPSProcessingMethod 
            | Tag::GPSAreaInformation | Tag::GPSDateStamp | Tag::GPSDifferential => {
                PrivacyCategory::Location
            }

            Tag::CameraSerialNumber | Tag::LensSerialNumber | Tag::BodySerialNumber 
            | Tag::InternalSerialNumber | Tag::UniqueCameraModel => {
                PrivacyCategory::DeviceIdentifier
            }

            Tag::CameraOwnerName | Tag::Artist | Tag::Copyright | Tag::UserComment 
            | Tag::XPTitle | Tag::XPComment | Tag::XPAuthor | Tag::XPKeywords | Tag::XPSubject => {
                PrivacyCategory::PersonalInfo
            }

            Tag::DateTime | Tag::DateTimeOriginal | Tag::DateTimeDigitized 
            | Tag::SubSecTime | Tag::SubSecTimeOriginal | Tag::SubSecTimeDigitized => {
                PrivacyCategory::Temporal
            }

            Tag::Software | Tag::ProcessingSoftware | Tag::HostComputer => {
                PrivacyCategory::Software
            }

            Tag::ImageDescription | Tag::DocumentName | Tag::PageName => {
                PrivacyCategory::Metadata
            }

            _ => PrivacyCategory::Other
        }
    }
}

impl Default for ExifAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PrivacyField {
    pub tag: exif::Tag,
    pub description: String,
    pub category: PrivacyCategory,
}

#[derive(Debug, Clone)]
pub struct ExifField {
    pub tag: exif::Tag,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrivacyCategory {
    Location,
    DeviceIdentifier,
    PersonalInfo,
    Temporal,
    Software,
    Metadata,
    Other,
}

impl std::fmt::Display for PrivacyCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrivacyCategory::Location => write!(f, "Location Data"),
            PrivacyCategory::DeviceIdentifier => write!(f, "Device Identifier"),
            PrivacyCategory::PersonalInfo => write!(f, "Personal Information"),
            PrivacyCategory::Temporal => write!(f, "Timestamp"),
            PrivacyCategory::Software => write!(f, "Software Information"),
            PrivacyCategory::Metadata => write!(f, "Metadata"),
            PrivacyCategory::Other => write!(f, "Other"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exif::Tag;

    #[test]
    fn test_privacy_field_categorization() {
        let analyzer = ExifAnalyzer::new();

        // Test GPS categorization
        assert_eq!(analyzer.categorize_privacy_field(Tag::GPSLatitude), PrivacyCategory::Location);
        assert_eq!(analyzer.categorize_privacy_field(Tag::GPSLongitude), PrivacyCategory::Location);

        // Test device identifier categorization
        assert_eq!(analyzer.categorize_privacy_field(Tag::CameraSerialNumber), PrivacyCategory::DeviceIdentifier);
        assert_eq!(analyzer.categorize_privacy_field(Tag::LensSerialNumber), PrivacyCategory::DeviceIdentifier);

        // Test personal info categorization
        assert_eq!(analyzer.categorize_privacy_field(Tag::Artist), PrivacyCategory::PersonalInfo);
        assert_eq!(analyzer.categorize_privacy_field(Tag::Copyright), PrivacyCategory::PersonalInfo);

        // Test temporal categorization
        assert_eq!(analyzer.categorize_privacy_field(Tag::DateTime), PrivacyCategory::Temporal);
        assert_eq!(analyzer.categorize_privacy_field(Tag::DateTimeOriginal), PrivacyCategory::Temporal);

        // Test software categorization
        assert_eq!(analyzer.categorize_privacy_field(Tag::Software), PrivacyCategory::Software);
        assert_eq!(analyzer.categorize_privacy_field(Tag::ProcessingSoftware), PrivacyCategory::Software);
    }

    #[test]
    fn test_has_exif_data_with_invalid_data() {
        let analyzer = ExifAnalyzer::new();
        let invalid_data = vec![0x00, 0x00, 0x00, 0x00];
        
        assert!(!analyzer.has_exif_data(&invalid_data));
    }

    #[test]
    fn test_analyze_privacy_data_no_exif() {
        let analyzer = ExifAnalyzer::new();
        let no_exif_data = vec![0xFF, 0xD8, 0xFF, 0xD9]; // Minimal JPEG without EXIF
        
        let result = analyzer.analyze_privacy_data(
            &no_exif_data, 
            Path::new("test.jpg"), 
            &PrivacyLevel::Standard, 
            false
        ).unwrap();
        
        assert!(result.is_empty());
    }
}