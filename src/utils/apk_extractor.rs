use crate::error::ApkExtractError;
use regex::Regex;
use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek},
};
use zip::{read::ZipFile, ZipArchive};

const HASH_REGEX_PATTERN: &str =
    r".+(\d\.\d\.\d).+([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})";
const ANDROID_CANDIDATE: &str = r"6350e2ec327334c8a9b7f494f344a761";

#[derive(Debug)]
pub struct AppInfo {
    pub version: Option<String>,
    pub hashes: Vec<String>,
}

/// Interface for extracting the app version and hash from a valid Global/JP server game apk/xapk.
///
/// Implementation Credit: https://github.com/mos9527/sssekai/blob/main/sssekai/entrypoint/apphash.py
pub struct ApkExtractor<T> {
    apk_buf: BufReader<T>,
}

impl ApkExtractor<File> {
    /// Creates a new ApkExtractor, loading the apk from the provided path.
    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        Ok(Self::new(reader))
    }
}

impl<T> ApkExtractor<T>
where
    T: Read + Seek,
{
    /// Creates a new ApkExtractor using the provided BufReader.
    pub fn new(apk_buf: BufReader<T>) -> Self {
        Self { apk_buf }
    }

    /// Extracts the app hash from the APK.
    pub fn extract(&mut self) -> Result<AppInfo, ApkExtractError> {
        let mut zip = ZipArchive::new(&mut self.apk_buf)?;

        // get the indexes where inner apks exist
        let inner_apk_indexes: Vec<usize> = zip
            .file_names()
            .enumerate()
            .filter(|(_, file_name)| file_name.ends_with(".apk"))
            .map(|(index, _)| index)
            .collect();

        // extract app hashes
        let hash_re = Regex::new(HASH_REGEX_PATTERN)?;

        // If no inner apk indexes were found, we might be dealing with a .apk instead of a .xapk.
        if inner_apk_indexes.is_empty() {
            extract_info_from_archive(&mut zip, &hash_re)
        } else {
            let mut version = None;
            let mut hashes = Vec::new();

            for apk_index in inner_apk_indexes {
                // read the apk file into a buffer
                let mut file = zip.by_index(apk_index)?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;

                // read this data as another zip file.
                let cursor = Cursor::new(buf);
                let mut inner_zip = ZipArchive::new(cursor)?;

                let extracted_info = extract_info_from_archive(&mut inner_zip, &hash_re)?;
                if version.is_none() {
                    version = extracted_info.version;
                }
                hashes.extend(extracted_info.hashes);
            }

            Ok(AppInfo { version, hashes })
        }
    }
}

/// Extracts app info from the provided ZipFile using the given regular expression
///
/// Returns a tuple where the first argument is the AppVersion
/// and the second argument is the AppHash
fn extract_file_app_info(
    file: &mut ZipFile<'_>,
    re: &Regex,
) -> Result<Option<(String, String)>, ApkExtractError> {
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf)?;

    let file_content = String::from_utf8_lossy(&file_buf);

    re.captures(&file_content)
        .and_then(|captures| {
            captures
                .get(1)
                .zip(captures.get(2))
                .map(|(version, hash)| (version.as_str().to_string(), hash.as_str().to_string()))
        })
        .map(Ok)
        .transpose()
}

/// Extracts app info from inside the provided archive.
fn extract_info_from_archive(
    archive: &mut ZipArchive<impl Read + Seek>,
    hash_re: &Regex,
) -> Result<AppInfo, ApkExtractError> {
    let mut version = None;
    let mut hashes: Vec<String> = Vec::new();

    // get candidates
    let candidates: Vec<usize> = archive
        .file_names()
        .enumerate()
        .filter(|(_, file_name)| file_name.ends_with(ANDROID_CANDIDATE))
        .map(|(index, _)| index)
        .collect();

    // extract the app hash from any candidates
    for candidate_index in candidates {
        let mut candidate_file = archive.by_index(candidate_index)?;
        if let Some((extract_version, hash)) = extract_file_app_info(&mut candidate_file, hash_re)?
        {
            if version.is_none() {
                version = Some(extract_version);
            }
            hashes.push(hash)
        }
    }

    Ok(AppInfo { version, hashes })
}
