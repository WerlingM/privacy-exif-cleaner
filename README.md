# Privacy EXIF Cleaner

A Rust tool for removing privacy-sensitive information from image EXIF data while preserving useful technical metadata. Protect your privacy by selectively removing GPS coordinates, device identifiers, timestamps, and personal information from your photos.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ Features

- **🔒 Multiple Privacy Levels**: Choose from minimal to paranoid data removal
- **📍 GPS Removal**: Strip location data while keeping camera settings
- **🔍 Selective Processing**: Remove only what you want, preserve what you need
- **💾 Backup Support**: Automatic backup creation for safety
- **📊 Detailed Analysis**: See exactly what privacy data exists before removal
- **🚀 Batch Processing**: Handle entire directories recursively
- **🔧 Library & CLI**: Use as a command-line tool or integrate into your Rust projects

## 🛡️ Privacy Levels

| Level | Removes | Preserves |
|-------|---------|-----------|
| **Minimal** | GPS coordinates, location data | All camera settings, timestamps, device info |
| **Standard** | GPS + device serial numbers + personal info | Camera settings, timestamps, technical data |
| **Strict** | GPS + device IDs + timestamps + software info | Camera settings, color profiles |
| **Paranoid** | Everything except essential camera settings | Only ISO, aperture, focal length, exposure time |

## 📋 Requirements

- **Rust 1.70+** (for building from source)
- **ExifTool** (required runtime dependency)

## 🚀 Installation

### Installing ExifTool

ExifTool is required for metadata manipulation. Install it first:

**macOS:**
```bash
brew install exiftool
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install libimage-exiftool-perl
```

**CentOS/RHEL/Fedora:**
```bash
# Fedora
sudo dnf install perl-Image-ExifTool

# CentOS/RHEL (enable EPEL first)
sudo yum install epel-release
sudo yum install perl-Image-ExifTool
```

**Windows:**
```bash
# Using Chocolatey
choco install exiftool

# Or download from https://exiftool.org/
```

**Verify installation:**
```bash
exiftool -ver
# Should output version number like "12.76"
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/privacy-exif-cleaner.git
cd privacy-exif-cleaner

# Build the project
cargo build --release

# The binary will be available at:
./target/release/privacy-exif-cleaner
```

### Installing via Cargo

```bash
# Install directly from source
cargo install --path .

# Or if published to crates.io
cargo install privacy-exif-cleaner
```

## 📖 Usage

### Command Line Interface

#### Basic Usage

```bash
# Remove GPS data from all images in a directory
privacy-exif-cleaner -i photos/ -p minimal

# Standard privacy cleaning with backups
privacy-exif-cleaner -i photos/ -p standard -b

# Recursive processing with verbose output
privacy-exif-cleaner -i photos/ -p standard -r -v

# Paranoid mode - keep only essential camera settings
privacy-exif-cleaner -i photos/ -p paranoid -o cleaned/
```

#### Dry Run Mode

See what would be removed without making changes:

```bash
# Analyze what privacy data exists
privacy-exif-cleaner -i photos/ -n -v

# Check specific privacy level impact
privacy-exif-cleaner -i photos/ -p strict -n -v
```

#### Command Line Options

```
OPTIONS:
    -i, --input <DIR>        Input directory containing images [REQUIRED]
    -o, --output <DIR>       Output directory (optional - modifies in-place if not specified)
    -p, --privacy <LEVEL>    Privacy level: minimal, standard, strict, paranoid [default: standard]
    -r, --recursive          Process subdirectories recursively
    -b, --backup             Create backup files with .bak extension
    -v, --verbose            Show detailed information about data being removed
    -n, --dry-run            Show what would be removed without making changes
    -h, --help               Print help information
    -V, --version            Print version information
```

### Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
privacy-exif-cleaner = "0.1.0"
```

#### Simple Operations

```rust
use privacy_exif_cleaner::convenience::*;

// Remove GPS data from a single image
let had_gps = remove_gps_data("photo.jpg")?;

// Remove standard privacy data
let had_privacy_data = remove_standard_privacy_data("photo.jpg")?;

// Check what privacy data exists
let summary = get_privacy_summary("photo.jpg", PrivacyLevel::Standard)?;
println!("Found {} privacy fields", summary.total_privacy_fields);

// Check if image has GPS data
if has_gps_data("photo.jpg")? {
    println!("Image contains GPS coordinates");
}
```

#### Advanced Usage

```rust
use privacy_exif_cleaner::{PrivacyExifCleaner, PrivacyLevel, Config};

// Create custom configuration
let config = Config {
    input_dir: "photos".to_string(),
    output_dir: Some("cleaned".to_string()),
    recursive: true,
    create_backup: true,
    privacy_level: PrivacyLevel::Strict,
    verbose: false,
    dry_run: false,
};

// Process with custom config
let cleaner = PrivacyExifCleaner::new(config);
let had_privacy_data = cleaner.process_image("photo.jpg")?;

// Analyze before processing
let privacy_fields = cleaner.analyze_image("photo.jpg")?;
for field in privacy_fields {
    println!("Found {}: {} ({})", field.tag, field.description, field.category);
}
```

## 🔍 Examples

### Basic Workflow

```bash
# 1. Analyze what privacy data exists
privacy-exif-cleaner -i vacation_photos/ -n -v

# Output:
# Privacy data found in vacation_photos/IMG_001.jpg: GPS Latitude: 40.7128 (Location Data)
# Privacy data found in vacation_photos/IMG_001.jpg: GPS Longitude: -74.0060 (Location Data)
# Privacy data found in vacation_photos/IMG_001.jpg: Camera Serial Number: ABC123 (Device Identifier)

# 2. Remove privacy data with backup
privacy-exif-cleaner -i vacation_photos/ -p standard -b -v

# 3. Verify removal
exiftool vacation_photos/IMG_001.jpg | grep GPS
# (Should show no GPS data)
```

### Batch Processing

```bash
# Process entire photo library
privacy-exif-cleaner -i ~/Pictures/ -o ~/Pictures_Cleaned/ -p standard -r -v

# Summary:
# Files processed: 247
# Files with privacy data found: 156
# Errors: 0
```

### Integration with Photography Workflow

```bash
# Before sharing photos online
privacy-exif-cleaner -i ready_to_share/ -p strict -v

# For social media (paranoid mode)
privacy-exif-cleaner -i social_media/ -p paranoid -o social_media_clean/

# Quick GPS removal
privacy-exif-cleaner -i photos/ -p minimal
```

## 🧪 Supported File Formats

Currently supports:
- **JPEG** (.jpg, .jpeg) - Full support
- **TIFF** (.tif, .tiff) - Limited support

**Note**: PNG, GIF, and other formats don't typically contain EXIF data, so they're not processed.

## ⚠️ Important Notes

### What Gets Removed

**Minimal Level:**
- GPS coordinates (latitude, longitude, altitude)
- GPS timestamps and satellite info
- Location processing methods

**Standard Level:**
- All minimal level items
- Camera and lens serial numbers
- Device unique identifiers
- Owner name, artist, copyright info
- User comments

**Strict Level:**
- All standard level items
- Timestamps (when photo was taken)
- Software processing information
- Image descriptions and metadata
- XMP and IPTC data

**Paranoid Level:**
- Everything except: ISO, aperture, focal length, exposure time, camera make/model, basic technical settings

### What's Always Preserved

Even in paranoid mode, these essential camera settings are kept:
- ISO sensitivity
- Aperture (f-stop)
- Shutter speed
- Focal length
- Camera make and model
- Basic image dimensions and orientation

### Backup Recommendations

- Always test on copies first
- Use `-b/--backup` flag for important photos
- Consider using `-o/--output` to write to a separate directory
- Verify results with `exiftool` before deleting originals

## 🛠️ Developer Instructions

### Setting Up Development Environment

```bash
# Clone and setup
git clone https://github.com/yourusername/privacy-exif-cleaner.git
cd privacy-exif-cleaner

# Install development dependencies
cargo build

# Install recommended tools
cargo install cargo-watch cargo-clippy cargo-fmt
```

### Project Structure

```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Library interface
├── cli.rs           # Command line parsing
├── privacy.rs       # Privacy policy engine
├── processor.rs     # Image processing coordinator
├── analyzer.rs      # EXIF analysis engine
├── remover.rs       # Metadata removal engine
└── utils.rs         # Utility functions
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with test output
cargo test -- --nocapture

# Run specific module tests
cargo test privacy::tests

# Run with ExifTool integration tests (requires ExifTool)
cargo test --ignored
```

### Development Workflow

```bash
# Watch for changes and run tests
cargo watch -x test

# Format code
cargo fmt

# Run linter
cargo clippy

# Check without building
cargo check
```

### Adding New Privacy Levels

1. **Add to enum** in `src/privacy.rs`:
```rust
#[derive(Clone, Debug, ValueEnum)]
pub enum PrivacyLevel {
    Minimal,
    Standard,
    Strict,
    Paranoid,
    YourNewLevel,  // Add here
}
```

2. **Update policy logic** in `get_tags_to_remove()`:
```rust
PrivacyLevel::YourNewLevel => {
    // Define what tags to remove
}
```

3. **Add ExifTool commands** in `src/remover.rs`:
```rust
fn add_your_new_level_args(&self, cmd: &mut Command) {
    // Add ExifTool arguments
}
```

4. **Update CLI help** in `src/cli.rs` and this README

### Adding New File Format Support

1. **Update file detection** in `src/utils.rs`:
```rust
pub fn is_supported_image(path: &Path) -> bool {
    // Add new extensions
}
```

2. **Add format-specific handling** in `src/analyzer.rs` if needed

3. **Test with sample files** of the new format

### Creating New Removal Backends

1. **Define trait** in `src/remover.rs`:
```rust
pub trait MetadataRemover {
    fn remove_privacy_data(&self, input: &Path, output: &Path, level: &PrivacyLevel) -> Result<(), Error>;
}
```

2. **Implement for new backend**:
```rust
pub struct NativeRustRemover;
impl MetadataRemover for NativeRustRemover { /* ... */ }
```

3. **Add backend selection** to `Config`

### Code Style Guidelines

- Use `cargo fmt` for formatting
- Follow Rust naming conventions
- Add documentation comments for public APIs
- Include unit tests for new functions
- Handle errors explicitly with `Result` types
- Use meaningful variable names

### Testing Guidelines

- **Unit tests**: Test individual functions in isolation
- **Integration tests**: Test module interactions
- **Property tests**: Use `proptest` for invariant testing
- **Real data tests**: Mark with `#[ignore]` for manual running

### Performance Considerations

- Process files one at a time (don't load entire directories)
- Use streaming I/O where possible
- Delegate heavy operations to ExifTool
- Profile with `cargo bench` for hot paths

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Run the test suite: `cargo test`
5. Format code: `cargo fmt`
6. Run linter: `cargo clippy`
7. Commit changes: `git commit -am 'Add new feature'`
8. Push to branch: `git push origin feature-name`
9. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [ExifTool](https://exiftool.org/) by Phil Harvey for the robust metadata manipulation
- [Rust EXIF Library](https://github.com/kamadak/exif-rs) for EXIF parsing
- The Rust community for excellent libraries and tools

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/privacy-exif-cleaner/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/privacy-exif-cleaner/discussions)
- **Documentation**: This README and inline code documentation

## 🔮 Roadmap

- [ ] Native Rust EXIF removal (remove ExifTool dependency)
- [ ] GUI interface
- [ ] Additional file format support (RAW files)
- [ ] Batch operation progress bars
- [ ] Configuration file support
- [ ] Plugin system for custom privacy policies
- [ ] Integration with photo management tools