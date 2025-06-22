use std::collections::HashSet;
use clap::ValueEnum;
use exif::Tag;

#[derive(Clone, Debug, ValueEnum)]
pub enum PrivacyLevel {
    /// Remove only location data (GPS)
    Minimal,
    /// Remove location + identifying device information
    Standard,
    /// Remove all potentially identifying information
    Strict,
    /// Remove everything except basic technical settings
    Paranoid,
}

pub struct PrivacyPolicy;

impl PrivacyPolicy {
    /// Get the set of EXIF tags that should be removed for a given privacy level
    pub fn get_tags_to_remove(privacy_level: &PrivacyLevel) -> HashSet<Tag> {
        let mut tags = HashSet::new();

        // Always remove GPS data (all privacy levels)
        tags.extend(Self::get_gps_tags());

        match privacy_level {
            PrivacyLevel::Minimal => {
                // Only GPS data removed above
            }
            PrivacyLevel::Standard => {
                tags.extend(Self::get_device_identifying_tags());
                tags.extend(Self::get_personal_info_tags());
            }
            PrivacyLevel::Strict => {
                tags.extend(Self::get_device_identifying_tags());
                tags.extend(Self::get_personal_info_tags());
                tags.extend(Self::get_temporal_tags());
                tags.extend(Self::get_software_tags());
                tags.extend(Self::get_metadata_tags());
            }
            PrivacyLevel::Paranoid => {
                // In paranoid mode, we use a whitelist approach
                // This is handled in should_preserve_tag()
            }
        }

        tags
    }

    /// Determine if a tag should be preserved (inverse of removal logic)
    pub fn should_preserve_tag(tag: Tag, privacy_level: &PrivacyLevel) -> bool {
        match privacy_level {
            PrivacyLevel::Paranoid => {
                // In paranoid mode, only preserve essential technical settings
                Self::is_essential_camera_setting(tag)
            }
            _ => {
                // For other levels, check if the tag is in the removal list
                !Self::get_tags_to_remove(privacy_level).contains(&tag)
            }
        }
    }

    /// GPS and location-related tags
    fn get_gps_tags() -> Vec<Tag> {
        vec![
            Tag::GPSVersionID,
            Tag::GPSLatitudeRef,
            Tag::GPSLatitude,
            Tag::GPSLongitudeRef,
            Tag::GPSLongitude,
            Tag::GPSAltitudeRef,
            Tag::GPSAltitude,
            Tag::GPSTimeStamp,
            Tag::GPSSatellites,
            Tag::GPSStatus,
            Tag::GPSMeasureMode,
            Tag::GPSDOP,
            Tag::GPSSpeedRef,
            Tag::GPSSpeed,
            Tag::GPSTrackRef,
            Tag::GPSTrack,
            Tag::GPSImgDirectionRef,
            Tag::GPSImgDirection,
            Tag::GPSMapDatum,
            Tag::GPSDestLatitudeRef,
            Tag::GPSDestLatitude,
            Tag::GPSDestLongitudeRef,
            Tag::GPSDestLongitude,
            Tag::GPSDestBearingRef,
            Tag::GPSDestBearing,
            Tag::GPSDestDistanceRef,
            Tag::GPSDestDistance,
            Tag::GPSProcessingMethod,
            Tag::GPSAreaInformation,
            Tag::GPSDateStamp,
            Tag::GPSDifferential,
        ]
    }

    /// Device identifying information
    fn get_device_identifying_tags() -> Vec<Tag> {
        vec![
            Tag::CameraSerialNumber,
            Tag::LensSerialNumber,
            Tag::BodySerialNumber,
            Tag::InternalSerialNumber,
            Tag::UniqueCameraModel,
        ]
    }

    /// Personal information tags
    fn get_personal_info_tags() -> Vec<Tag> {
        vec![
            Tag::CameraOwnerName,
            Tag::Artist,
            Tag::Copyright,
            Tag::UserComment,
        ]
    }

    /// Temporal/timestamp tags
    fn get_temporal_tags() -> Vec<Tag> {
        vec![
            Tag::DateTime,
            Tag::DateTimeOriginal,
            Tag::DateTimeDigitized,
            Tag::SubSecTime,
            Tag::SubSecTimeOriginal,
            Tag::SubSecTimeDigitized,
        ]
    }

    /// Software and processing tags
    fn get_software_tags() -> Vec<Tag> {
        vec![
            Tag::Software,
            Tag::ProcessingSoftware,
            Tag::HostComputer,
        ]
    }

    /// Additional metadata tags
    fn get_metadata_tags() -> Vec<Tag> {
        vec![
            Tag::ImageDescription,
            Tag::DocumentName,
            Tag::PageName,
            Tag::XPTitle,
            Tag::XPComment,
            Tag::XPAuthor,
            Tag::XPKeywords,
            Tag::XPSubject,
        ]
    }

    /// Essential camera settings that should be preserved even in paranoid mode
    fn is_essential_camera_setting(tag: Tag) -> bool {
        matches!(
            tag,
            Tag::ExposureTime
                | Tag::FNumber
                | Tag::ISO
                | Tag::ISOSpeedRatings
                | Tag::FocalLength
                | Tag::FocalLengthIn35mmFilm
                | Tag::ExposureProgram
                | Tag::MeteringMode
                | Tag::Flash
                | Tag::ColorSpace
                | Tag::WhiteBalance
                | Tag::ExposureMode
                | Tag::SceneCaptureType
                | Tag::Contrast
                | Tag::Saturation
                | Tag::Sharpness
                | Tag::Make
                | Tag::Model // Keep camera make/model but not serial numbers
                | Tag::Orientation
                | Tag::XResolution
                | Tag::YResolution
                | Tag::ResolutionUnit
                | Tag::YCbCrPositioning
                | Tag::ExifVersion
                | Tag::ComponentsConfiguration
                | Tag::CompressedBitsPerPixel
                | Tag::PixelXDimension
                | Tag::PixelYDimension
        )
    }

    /// Get a human-readable description of what each privacy level removes
    pub fn get_privacy_description(level: &PrivacyLevel) -> Vec<&'static str> {
        match level {
            PrivacyLevel::Minimal => vec!["GPS coordinates", "location data"],
            PrivacyLevel::Standard => vec![
                "GPS data",
                "camera serial numbers",
                "unique device IDs",
                "personal information",
            ],
            PrivacyLevel::Strict => vec![
                "GPS data",
                "device identifiers",
                "timestamps",
                "user comments",
                "software information",
                "additional metadata",
            ],
            PrivacyLevel::Paranoid => vec!["all metadata except essential camera settings"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_level_escalation() {
        let minimal_tags = PrivacyPolicy::get_tags_to_remove(&PrivacyLevel::Minimal);
        let standard_tags = PrivacyPolicy::get_tags_to_remove(&PrivacyLevel::Standard);
        let strict_tags = PrivacyPolicy::get_tags_to_remove(&PrivacyLevel::Strict);

        // Each level should include more tags than the previous
        assert!(standard_tags.len() > minimal_tags.len());
        assert!(strict_tags.len() > standard_tags.len());

        // All levels should include GPS tags
        assert!(minimal_tags.contains(&Tag::GPSLatitude));
        assert!(standard_tags.contains(&Tag::GPSLatitude));
        assert!(strict_tags.contains(&Tag::GPSLatitude));
    }

    #[test]
    fn test_paranoid_preservation() {
        // Paranoid mode should preserve essential camera settings
        assert!(PrivacyPolicy::should_preserve_tag(Tag::ISO, &PrivacyLevel::Paranoid));
        assert!(PrivacyPolicy::should_preserve_tag(Tag::FNumber, &PrivacyLevel::Paranoid));
        assert!(PrivacyPolicy::should_preserve_tag(Tag::ExposureTime, &PrivacyLevel::Paranoid));

        // But not personal info
        assert!(!PrivacyPolicy::should_preserve_tag(Tag::Artist, &PrivacyLevel::Paranoid));
        assert!(!PrivacyPolicy::should_preserve_tag(Tag::GPSLatitude, &PrivacyLevel::Paranoid));
    }

    #[test]
    fn test_gps_coverage() {
        let minimal_tags = PrivacyPolicy::get_tags_to_remove(&PrivacyLevel::Minimal);
        
        // Ensure key GPS tags are covered
        let important_gps_tags = [
            Tag::GPSLatitude,
            Tag::GPSLongitude,
            Tag::GPSAltitude,
            Tag::GPSTimeStamp,
        ];

        for tag in important_gps_tags.iter() {
            assert!(minimal_tags.contains(tag), "GPS tag {:?} not covered by minimal privacy", tag);
        }
    }

    #[test]
    fn test_essential_camera_settings() {
        let essential_tags = [
            Tag::ISO,
            Tag::FNumber,
            Tag::ExposureTime,
            Tag::FocalLength,
            Tag::Make,
            Tag::Model,
        ];

        for tag in essential_tags.iter() {
            assert!(PrivacyPolicy::is_essential_camera_setting(*tag), 
                    "Tag {:?} should be considered essential camera setting", tag);
        }

        // These should NOT be essential
        let non_essential_tags = [
            Tag::GPSLatitude,
            Tag::Artist,
            Tag::CameraSerialNumber,
            Tag::DateTime,
        ];

        for tag in non_essential_tags.iter() {
            assert!(!PrivacyPolicy::is_essential_camera_setting(*tag), 
                    "Tag {:?} should NOT be considered essential camera setting", tag);
        }
    }
}