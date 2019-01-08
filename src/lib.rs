use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

use failure::Fail;
use log;

#[derive(Debug, Fail)]
pub enum DirectoryLinesStreamerError {
    #[fail(display = "directory {:?} does not exists", _0)]
    DirectoryDoesNotExists(PathBuf),
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "directory {:?} is empty", _0)]
    EmptyDirectory(PathBuf),
}

#[derive(Debug)]
pub struct DirectoryLinesStreamer {
    dir: PathBuf,
    files: Vec<PathBuf>,
    opened_file_path: PathBuf,
    opened_file: BufReader<File>,
    line_id: usize,
}

impl DirectoryLinesStreamer {
    pub fn from_dir<P>(input_dir: P) -> Result<DirectoryLinesStreamer, failure::Error>
    where
        P: Into<PathBuf>,
    {
        let dir = input_dir.into();
        if dir.exists() {
            let dir_entries = fs::read_dir(&dir)?;

            let mut files: Vec<PathBuf> = dir_entries
                // Keep only valid entries
                .filter_map(Result::ok)
                // Convert to a PathBuf
                .map(|dir_entry| dir_entry.path())
                // Collect into a Vec<_>
                .collect();
            // Sort the files using the `alphanumeric_sort` crate, which will place `file-2` before `file-11`.
            alphanumeric_sort::sort_path_slice(&mut files);
            // We'll `pop()` the last file until we are done, so we want to invert the vec.
            let mut files: Vec<PathBuf> = files.into_iter().rev().collect();
            log::debug!("files: {:?}", files);

            // Open the first file
            if files.is_empty() {
                Err(DirectoryLinesStreamerError::EmptyDirectory(dir).into())
            } else {
                // Safe since we verified to contain at least one file
                let opened_file_path = files.pop().unwrap();

                log::debug!("Opening first file: {:?}", opened_file_path);
                let opened_file = BufReader::new(File::open(&opened_file_path)?);

                Ok(DirectoryLinesStreamer {
                    dir,
                    files,
                    opened_file_path,
                    opened_file,
                    line_id: 1,
                })
            }
        } else {
            Err(DirectoryLinesStreamerError::DirectoryDoesNotExists(dir).into())
        }
    }
}

impl Iterator for DirectoryLinesStreamer {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        read_next_line_from_files(
            &mut self.files,
            &mut self.opened_file,
            &mut self.opened_file_path,
            &mut self.line_id,
        )
    }
}

fn read_next_line_from_files(
    files: &mut Vec<PathBuf>,
    opened_file: &mut BufReader<File>,
    opened_file_path: &mut PathBuf,
    line_id: &mut usize,
) -> Option<String> {
    loop {
        let line = read_line_from_file(opened_file, opened_file_path, *line_id);
        *line_id += 1;
        if line.is_some() {
            return line;
        } else {
            // EOF reached. Try next file
            let next_file = files.pop()?;
            log::debug!("Opening next file: {:?}", next_file);
            if let Ok(f) = File::open(&next_file)
                .map_err(|e| log::error!("Error opening file {:?}: {:?}", next_file, e))
            {
                *opened_file = BufReader::new(f);
                *opened_file_path = next_file;
            }
        }
    }
}

fn read_line_from_file(
    f: &mut BufReader<File>,
    file_path: &PathBuf,
    line_id: usize,
) -> Option<String> {
    let mut buf: Vec<u8> = Vec::new();
    // Read bytes until a newline character is found (0xA).
    let nb_bytes_read_result = f.read_until(b'\n', &mut buf);
    // Convert to UTF-8 to get a string, replacing bad characters
    // with U+FFFD REPLACEMENT CHARACTER (`�`)
    let line = String::from_utf8_lossy(&buf).to_string();

    match nb_bytes_read_result {
        Ok(nb_bytes_read) => {
            if nb_bytes_read == 0 {
                // EOF reached
                None
            } else {
                Some(line)
            }
        }
        Err(e) => {
            // I/O errors happened. Report it and continue.
            log::error!("Error reading line {} of {:?}: {:?}", line_id, file_path, e);
            Some(line)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streamer_failure() {
        match DirectoryLinesStreamer::from_dir("fixtures/non-existent-dir")
            .unwrap_err()
            .downcast_ref()
            .unwrap()
        {
            DirectoryLinesStreamerError::DirectoryDoesNotExists(dir) => {
                assert_eq!(dir, &PathBuf::from("fixtures/non-existent-dir"))
            }
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn streamer_empty() {
        match DirectoryLinesStreamer::from_dir("fixtures/empty-dir")
            .unwrap_err()
            .downcast_ref()
            .unwrap()
        {
            DirectoryLinesStreamerError::EmptyDirectory(dir) => {
                assert_eq!(dir, &PathBuf::from("fixtures/empty-dir"))
            }
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn streamer_success() {
        let streamer = DirectoryLinesStreamer::from_dir("fixtures/non-empty-dir").unwrap();
        let expected_lines = &[
            "line one from messages\n",
            "line two from messages\n",
            "line three from messages\n",
            "line one from messages.1\n",
            "line two from messages.1\n",
            "line three from messages.1\n",
            "line one from messages.2\n",
            "line two from messages.2\n",
            "line three from messages.2\n",
            "line one from messages.10\n",
            "line two from messages.10\n",
            "line three from messages.10\n",
            "line one from messages.20\n",
            "line two from messages.20\n",
            "line three from messages.20\n",
        ];
        for (ref line, expected_line) in streamer.zip(expected_lines) {
            assert_eq!(line, expected_line);
        }
    }

    #[test]
    fn collect() {
        let streamer = DirectoryLinesStreamer::from_dir("fixtures/non-empty-dir").unwrap();
        let lines: Vec<String> = streamer.collect();

        let expected_lines = &[
            "line one from messages\n",
            "line two from messages\n",
            "line three from messages\n",
            "line one from messages.1\n",
            "line two from messages.1\n",
            "line three from messages.1\n",
            "line one from messages.2\n",
            "line two from messages.2\n",
            "line three from messages.2\n",
            "line one from messages.10\n",
            "line two from messages.10\n",
            "line three from messages.10\n",
            "line one from messages.20\n",
            "line two from messages.20\n",
            "line three from messages.20\n",
        ];
        for (ref line, expected_line) in lines.iter().zip(expected_lines) {
            assert_eq!(line, expected_line);
        }
    }
}
