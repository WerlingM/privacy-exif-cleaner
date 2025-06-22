use std::path::Path;
use std::process::Command;
use crate::privacy::PrivacyLevel;

pub struct MetadataRemover;

impl MetadataRemover {
    pub fn new() -> Self {
        Self
    }

    /// Remove privacy data from an image using ExifTool
    pub fn remove_privacy_data(
        &self,
        input_path: &Path,
        output_path: &Path,
        privacy_level: &PrivacyLevel,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check if ExifTool is available
        self.check_exiftool_availability()?;

        // Build and execute the ExifTool command
        let mut cmd = self.build_exiftool_command(privacy_level);
        
        // Configure input/output
        if input_path != output_path {
            // Writing to different file
            cmd.arg("-o").arg(output_path);
        } else {
            // In-place modification
            cmd.arg("-overwrite_original");
        }

        cmd.arg(input_path);

        // Execute the command
        let output = cmd.output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ExifTool failed: {}", stderr).into());
        }

        Ok(())
    }

    /// Check if ExifTool is installed and accessible
    fn check_exiftool_availability(&self) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("exiftool")
            .arg("-ver")
            .output();

        match output {
            Ok(output) if output.status.success() => Ok(()),
            Ok(_) => Err("ExifTool found but returned error".into()),
            Err(_) => Err("ExifTool not found. Please install ExifTool and ensure it's in your PATH".into()),
        }
    }

    /// Build the ExifTool command based on privacy level
    fn build_exiftool_command(&self, privacy_level: &PrivacyLevel) -> Command {
        let mut cmd = Command::new("exiftool");
        
        match privacy_level {
            PrivacyLevel::Minimal => {
                self.add_minimal_removal_args(&mut cmd);
            }
            PrivacyLevel::Standard => {
                self.add_standard_removal_args(&mut cmd);
            }
            PrivacyLevel::Strict => {
                self.add_strict_removal_args(&mut cmd);
            }
            PrivacyLevel::Paranoid => {
                self.add_paranoid_removal_args(&mut cmd);
            }
        }

        cmd
    }

    /// Add arguments for minimal privacy (GPS only)
    fn add_minimal_removal_args(&self, cmd: &mut Command) {
        cmd.arg("-gps:all=");
    }

    /// Add arguments for standard privacy
    fn add_standard_removal_args(&self, cmd: &mut Command) {
        cmd.arg("-gps:all=")
           .arg("-SerialNumber=")
           .arg("-InternalSerialNumber=")
           .arg("-LensSerialNumber=")
           .arg("-CameraOwnerName=")
           .arg("-Artist=")
           .arg("-Copyright=")
           .arg("-UserComment=");
    }

    /// Add arguments for strict privacy
    fn add_strict_removal_args(&self, cmd: &mut Command) {
        // Include all standard removals
        self.add_standard_removal_args(cmd);
        
        // Add additional strict removals
        cmd.arg("-DateTime=")
           .arg("-DateTimeOriginal=")
           .arg("-DateTimeDigitized=")
           .arg("-Software=")
           .arg("-ProcessingSoftware=")
           .arg("-HostComputer=")
           .arg("-ImageDescription=")
           .arg("-XMP:all=")
           .arg("-IPTC:all=");
    }

    /// Add arguments for paranoid privacy (preserve only essential camera settings)
    fn add_paranoid_removal_args(&self, cmd: &mut Command) {
        // Remove all EXIF data first
        cmd.arg("-all=");

        // Then restore only essential camera settings
        cmd.arg("-TagsFromFile").arg("@")
           .arg("-ExposureTime")
           .arg("-FNumber")
           .arg("-ISO")
           .arg("-ISOSpeedRatings")
           .arg("-FocalLength")
           .arg("-FocalLengthIn35mmFilm")
           .arg("-ExposureProgram")
           .arg("-MeteringMode")
           .arg("-Flash")
           .arg("-ColorSpace")
           .arg("-WhiteBalance")
           .arg("-ExposureMode")
           .arg("-SceneCaptureType")
           .arg("-Contrast")
           .arg("-Saturation")
           .arg("-Sharpness")
           .arg("-Make")
           .arg("-Model")
           .arg("-Orientation")
           .arg("-XResolution")
           .arg("-YResolution")
           .arg("-ResolutionUnit")
           .arg("-ExifVersion")
           .arg("-ComponentsConfiguration")
           .arg("-PixelXDimension")
           .arg("-PixelYDimension");
    }

    /// Get the ExifTool version (for diagnostics)
    pub fn get_exiftool_version(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("exiftool")
            .arg("-ver")
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?.trim().to_string())
        } else {
            Err("Failed to get ExifTool version".into())
        }
    }

    /// Test ExifTool with a simple operation (for validation)
    pub fn test_exiftool_operation(&self, test_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("exiftool")
            .arg("-j")  // JSON output
            .arg("-q")  // Quiet mode
            .arg(test_file)
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("ExifTool test failed: {}", stderr).into())
        }
    }
}

impl Default for MetadataRemover {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn test_minimal_command_building() {
        let remover = MetadataRemover::new();
        let cmd = remover.build_exiftool_command(&PrivacyLevel::Minimal);
        
        // Convert command to string for testing
        let cmd_str = format!("{:?}", cmd);
        assert!(cmd_str.contains("-gps:all="));
    }

    #[test]
    fn test_standard_command_building() {
        let remover = MetadataRemover::new();
        let cmd = remover.build_exiftool_command(&PrivacyLevel::Standard);
        
        let cmd_str = format!("{:?}", cmd);
        assert!(cmd_str.contains("-gps:all="));
        assert!(cmd_str.contains("-SerialNumber="));
        assert!(cmd_str.contains("-Artist="));
    }

    #[test]
    fn test_strict_command_building() {
        let remover = MetadataRemover::new();
        let cmd = remover.build_exiftool_command(&PrivacyLevel::Strict);
        
        let cmd_str = format!("{:?}", cmd);
        assert!(cmd_str.contains("-gps:all="));
        assert!(cmd_str.contains("-DateTime="));
        assert!(cmd_str.contains("-Software="));
        assert!(cmd_str.contains("-XMP:all="));
    }

    #[test]
    fn test_paranoid_command_building() {
        let remover = MetadataRemover::new();
        let cmd = remover.build_exiftool_command(&PrivacyLevel::Paranoid);
        
        let cmd_str = format!("{:?}", cmd);
        assert!(cmd_str.contains("-all="));
        assert!(cmd_str.contains("-TagsFromFile"));
        assert!(cmd_str.contains("-ISO"));
        assert!(cmd_str.contains("-FNumber"));
    }

    #[test]
    fn test_exiftool_availability_check() {
        let remover = MetadataRemover::new();
        
        // This test will pass if ExifTool is installed, skip if not
        if let Ok(_) = remover.check_exiftool_availability() {
            // ExifTool is available
            assert!(true);
        } else {
            // ExifTool not available - this is expected in some test environments
            println!("Warning: ExifTool not available for testing");
        }
    }

    #[test]
    #[ignore] // Run only when ExifTool is definitely available
    fn test_exiftool_version() {
        let remover = MetadataRemover::new();
        let version = remover.get_exiftool_version();
        
        match version {
            Ok(ver) => {
                assert!(!ver.is_empty());
                // Version should be numeric (like "12.76")
                assert!(ver.chars().any(|c| c.is_ascii_digit()));
            }
            Err(_) => {
                panic!("ExifTool should be available for this test");
            }
        }
    }
}