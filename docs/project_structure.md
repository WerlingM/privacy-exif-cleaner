# Modular Privacy EXIF Cleaner - Project Structure

## Directory Layout

```
privacy-exif-cleaner/
├── Cargo.toml                 # Project configuration and dependencies
├── src/
│   ├── main.rs               # CLI entry point and orchestration
│   ├── lib.rs                # Library interface for external use
│   ├── cli.rs                # Command-line argument parsing
│   ├── privacy.rs            # Privacy policy engine
│   ├── processor.rs          # Image processing coordinator
│   ├── analyzer.rs           # EXIF analysis engine
│   ├── remover.rs            # Metadata removal engine
│   └── utils.rs              # Utility functions
├── tests/                    # Integration tests (to be created)
├── examples/                 # Usage examples (to be created)
└── README.md                 # Project documentation
```

## Module Responsibilities

### `main.rs` - Application Entry Point
- **Purpose**: CLI application orchestration
- **Responsibilities**:
  - Parse command-line arguments via `cli` module
  - Coordinate file discovery using `walkdir`
  - Drive the processing pipeline
  - Handle top-level error reporting and statistics
- **Dependencies**: All other modules
- **Key Functions**: `main()`, `run_processing()`, `print_summary()`

### `lib.rs` - Library Interface
- **Purpose**: Public API for using as a Rust library
- **Responsibilities**:
  - Re-export main types for easy access
  - Provide high-level convenience functions
  - Define library-level abstractions
- **Key Types**: `PrivacyExifCleaner`, `PrivacySummary`
- **Convenience Functions**: `remove_gps_data()`, `has_gps_data()`, etc.

### `cli.rs` - Command Line Interface
- **Purpose**: Argument parsing and configuration management
- **Responsibilities**:
  - Define CLI arguments using `clap`
  - Parse and validate user input
  - Create configuration objects
  - Provide help text and usage information
- **Key Types**: `Config` struct
- **Dependencies**: `clap`, `privacy` module for `PrivacyLevel`

### `privacy.rs` - Privacy Policy Engine
- **Purpose**: Core privacy logic and policy definitions
- **Responsibilities**:
  - Define privacy levels and their implications
  - Maintain lists of privacy-sensitive EXIF tags
  - Implement tag filtering logic (blacklist vs whitelist)
  - Provide policy descriptions for user education
- **Key Types**: `PrivacyLevel` enum, `PrivacyPolicy` struct
- **Key Functions**: `get_tags_to_remove()`, `should_preserve_tag()`

### `processor.rs` - Image Processing Coordinator
- **Purpose**: High-level image processing workflow
- **Responsibilities**:
  - Coordinate between analysis and removal phases
  - Handle file I/O operations
  - Manage backup creation
  - Determine output paths
  - Error handling for individual files
- **Key Types**: `ImageProcessor` struct
- **Dependencies**: `analyzer`, `remover`, `cli` modules

### `analyzer.rs` - EXIF Analysis Engine
- **Purpose**: EXIF data parsing and privacy analysis
- **Responsibilities**:
  - Parse EXIF data from image files
  - Identify privacy-sensitive fields
  - Categorize privacy violations
  - Provide detailed analysis reports
- **Key Types**: `ExifAnalyzer`, `PrivacyField`, `PrivacyCategory`
- **Dependencies**: `exif` crate, `privacy` module

### `remover.rs` - Metadata Removal Engine
- **Purpose**: Actual metadata removal implementation
- **Responsibilities**:
  - Interface with ExifTool
  - Build appropriate ExifTool commands
  - Handle different privacy levels
  - Validate ExifTool availability
  - Execute metadata removal operations
- **Key Types**: `MetadataRemover` struct
- **Dependencies**: `std::process::Command`, `privacy` module

### `utils.rs` - Utility Functions
- **Purpose**: Common utility functions and helpers
- **Responsibilities**:
  - File type detection and validation
  - Directory operations and permissions
  - File size formatting
  - Progress tracking
  - Error collection and reporting
- **Key Types**: `ProgressTracker`, `ErrorCollector`, `FileInfo`
- **No external dependencies** (except standard library)

## Data Flow Architecture

```
User Input (CLI) → Config → ImageProcessor
                                ↓
File Discovery → Supported Files → Process Each File
                                       ↓
                                   ExifAnalyzer
                                       ↓
                              Privacy Fields Found?
                                    ↓     ↓
                                  No     Yes
                                  ↓       ↓
                                Skip   MetadataRemover
                                         ↓
                                   ExifTool Execution
                                         ↓
                                   Updated File
```

## Module Interaction Patterns

### 1. **Dependency Injection Pattern**
- `ImageProcessor` receives `Config` and creates `ExifAnalyzer` and `MetadataRemover`
- Enables easy testing and configuration changes

### 2. **Strategy Pattern**
- `PrivacyLevel` enum drives different removal strategies
- Each level defines different tag removal behavior

### 3. **Pipeline Pattern**
- Files flow through: Discovery → Analysis → Removal → Reporting
- Each stage can fail independently without affecting others

### 4. **Facade Pattern**
- `lib.rs` provides simplified interface hiding internal complexity
- Convenience functions abstract common use cases

## Testing Strategy by Module

### Unit Tests (within each module)
- **`privacy.rs`**: Policy logic, tag categorization
- **`analyzer.rs`**: EXIF parsing, field categorization  
- **`remover.rs`**: Command building, ExifTool integration
- **`processor.rs`**: File operations, path handling
- **`utils.rs`**: File utilities, progress tracking
- **`cli.rs`**: Argument parsing, config validation

### Integration Tests (separate `tests/` directory)
- End-to-end workflow testing
- Real image file processing
- ExifTool integration validation
- Error handling scenarios

### Property-Based Tests
- Privacy level monotonicity (higher levels remove more data)
- Tag preservation guarantees
- File integrity preservation

## Extension Points

### Adding New Privacy Levels
1. Add variant to `PrivacyLevel` enum in `privacy.rs`
2. Update `get_tags_to_remove()` logic
3. Add ExifTool command logic in `remover.rs`
4. Update CLI help text in `cli.rs`

### Supporting New File Formats
1. Update `is_supported_image()` in `utils.rs`
2. Add format-specific handling in `analyzer.rs`
3. Update ExifTool commands in `remover.rs` if needed

### Alternative Removal Backends
1. Create new trait in `remover.rs`
2. Implement trait for different backends (native Rust, ImageMagick, etc.)
3. Add backend selection to `Config`

### Adding New Analysis Features
1. Extend `PrivacyCategory` enum in `analyzer.rs`
2. Add categorization logic in `categorize_privacy_field()`
3. Update summary reporting in `lib.rs`

## Key Design Principles

### 1. **Separation of Concerns**
Each module has a single, well-defined responsibility

### 2. **Testability**
Pure functions where possible, dependency injection for complex objects

### 3. **Error Handling**
- Library functions return `Result` types
- CLI application collects errors and continues processing
- Detailed error messages with context

### 4. **Performance**
- Streaming file processing (no large directories in memory)
- Minimal memory footprint per image
- External tool delegation for heavy lifting

### 5. **Extensibility**
- Enum-driven strategies for easy addition of new features
- Trait-based abstractions for swappable implementations
- Clear module boundaries for independent evolution

This modular structure makes the code much easier to understand, test, maintain, and extend compared to the original monolithic version.