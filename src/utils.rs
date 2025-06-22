use std::path::Path;

/// Check if a file is a supported image format
pub fn is_supported_image(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "jpg" | "jpeg" | "tif" | "tiff")
    } else {
        false
    }
}

/// Get a human-readable file size string
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Validate that a directory exists and is readable
pub fn validate_directory(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Directory '{}' does not exist", path.display()));
    }

    if !path.is_dir() {
        return Err(format!("Path '{}' is not a directory", path.display()));
    }

    // Try to read the directory to check permissions
    match std::fs::read_dir(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Cannot read directory '{}': {}", path.display(), e)),
    }
}

/// Check if a file is readable
pub fn is_file_readable(path: &Path) -> bool {
    std::fs::File::open(path).is_ok()
}

/// Get the extension of a file as a lowercase string
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Create a safe filename by replacing problematic characters
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

/// Get file metadata information
pub fn get_file_info(path: &Path) -> Result<FileInfo, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    
    Ok(FileInfo {
        size: metadata.len(),
        is_readonly: metadata.permissions().readonly(),
        modified: metadata.modified().ok(),
    })
}

#[derive(Debug)]
pub struct FileInfo {
    pub size: u64,
    pub is_readonly: bool,
    pub modified: Option<std::time::SystemTime>,
}

/// Check if we have write permission to a directory
pub fn can_write_to_directory(path: &Path) -> bool {
    // Try to create a temporary file
    let temp_file = path.join(".temp_write_test");
    
    match std::fs::write(&temp_file, b"test") {
        Ok(_) => {
            // Clean up the test file
            let _ = std::fs::remove_file(&temp_file);
            true
        }
        Err(_) => false,
    }
}

/// Progress tracking utility
#[derive(Debug, Default)]
pub struct ProgressTracker {
    total: u64,
    processed: u64,
    errors: u64,
}

impl ProgressTracker {
    pub fn new(total: u64) -> Self {
        Self {
            total,
            processed: 0,
            errors: 0,
        }
    }

    pub fn increment_processed(&mut self) {
        self.processed += 1;
    }

    pub fn increment_errors(&mut self) {
        self.errors += 1;
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            ((self.processed + self.errors) as f64 / self.total as f64) * 100.0
        }
    }

    pub fn remaining(&self) -> u64 {
        self.total.saturating_sub(self.processed + self.errors)
    }

    pub fn processed(&self) -> u64 {
        self.processed
    }

    pub fn errors(&self) -> u64 {
        self.errors
    }

    pub fn total(&self) -> u64 {
        self.total
    }
}

/// Simple error aggregation for batch operations
#[derive(Debug, Default)]
pub struct ErrorCollector {
    errors: Vec<(String, String)>, // (file_path, error_message)
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error<P: AsRef<Path>>(&mut self, file_path: P, error: &str) {
        self.errors.push((
            file_path.as_ref().display().to_string(),
            error.to_string(),
        ));
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn print_summary(&self) {
        if self.has_errors() {
            println!("\nErrors encountered:");
            for (path, error) in &self.errors {
                println!("  {}: {}", path, error);
            }
        }
    }

    pub fn get_errors(&self) -> &[(String, String)] {
        &self.errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_supported_image() {
        assert!(is_supported_image(Path::new("test.jpg")));
        assert!(is_supported_image(Path::new("test.jpeg")));
        assert!(is_supported_image(Path::new("test.JPEG")));
        assert!(is_supported_image(Path::new("test.tiff")));
        assert!(is_supported_image(Path::new("photo.TIF")));
        
        assert!(!is_supported_image(Path::new("test.png")));
        assert!(!is_supported_image(Path::new("test.gif")));
        assert!(!is_supported_image(Path::new("test.txt")));
        assert!(!is_supported_image(Path::new("test")));
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension(Path::new("test.jpg")), Some("jpg".to_string()));
        assert_eq!(get_file_extension(Path::new("test.JPEG")), Some("jpeg".to_string()));
        assert_eq!(get_file_extension(Path::new("test")), None);
        assert_eq!(get_file_extension(Path::new("test.")), Some("".to_string()));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal_file.jpg"), "normal_file.jpg");
        assert_eq!(sanitize_filename("file/with\\bad:chars*?.jpg"), "file_with_bad_chars__.jpg");
        assert_eq!(sanitize_filename("file<with>more|bad\"chars.jpg"), "file_with_more_bad_chars.jpg");
    }

    #[test]
    fn test_validate_directory() {
        let temp_dir = TempDir::new().unwrap();
        
        // Valid directory
        assert!(validate_directory(temp_dir.path()).is_ok());
        
        // Non-existent directory
        let non_existent = temp_dir.path().join("does_not_exist");
        assert!(validate_directory(&non_existent).is_err());
        
        // File instead of directory
        let temp_file = temp_dir.path().join("test_file.txt");
        fs::write(&temp_file, "test").unwrap();
        assert!(validate_directory(&temp_file).is_err());
    }

    #[test]
    fn test_progress_tracker() {
        let mut tracker = ProgressTracker::new(100);
        
        assert_eq!(tracker.progress_percentage(), 0.0);
        assert_eq!(tracker.remaining(), 100);
        
        tracker.increment_processed();
        assert_eq!(tracker.processed(), 1);
        assert_eq!(tracker.progress_percentage(), 1.0);
        assert_eq!(tracker.remaining(), 99);
        
        tracker.increment_errors();
        assert_eq!(tracker.errors(), 1);
        assert_eq!(tracker.progress_percentage(), 2.0);
        assert_eq!(tracker.remaining(), 98);
    }

    #[test]
    fn test_error_collector() {
        let mut collector = ErrorCollector::new();
        
        assert!(!collector.has_errors());
        assert_eq!(collector.error_count(), 0);
        
        collector.add_error(Path::new("file1.jpg"), "Test error 1");
        collector.add_error(Path::new("file2.jpg"), "Test error 2");
        
        assert!(collector.has_errors());
        assert_eq!(collector.error_count(), 2);
        
        let errors = collector.get_errors();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].0, "file1.jpg");
        assert_eq!(errors[0].1, "Test error 1");
    }

    #[test]
    fn test_can_write_to_directory() {
        let temp_dir = TempDir::new().unwrap();
        
        // Should be able to write to temp directory
        assert!(can_write_to_directory(temp_dir.path()));
        
        // Non-existent directory should return false
        let non_existent = temp_dir.path().join("does_not_exist");
        assert!(!can_write_to_directory(&non_existent));
    }

    #[test]
    fn test_get_file_info() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let test_content = b"Hello, world!";
        
        fs::write(&test_file, test_content).unwrap();
        
        let file_info = get_file_info(&test_file).unwrap();
        assert_eq!(file_info.size, test_content.len() as u64);
        assert!(file_info.modified.is_some());
    }
}