use crate::source::Source;

enum ValidFileExtension {
    WAV,
    MP3,
}

struct FileSource {
    source_path: String,
    file_name: String,
    file_extension: ValidFileExtension,
    file_size_bytes: u64,
}

impl Source for FileSource {}
