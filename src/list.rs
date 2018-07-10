use mime_guess::guess_mime_type_opt;
use rocket::response::Failure;
use rocket_contrib::Template;
use rocket::http::Status;
use pretty_bytes::converter::convert;
use std::ffi::OsStr;
use std::fs::{read_dir, Metadata, DirEntry};
use std::path::{Path, StripPrefixError};
use std::time::SystemTime;
use chrono::DateTime;
use chrono::offset::Utc;
pub use directory_path::DirectoryPath;

#[derive(Serialize)]
struct TemplateFile {
    name: String,
    size: String,
    file_type: String,
    mtime: String,
    is_dir: bool,
    is_symlink: bool,
    path: String,
    ext: String
}

#[derive(Serialize)]
struct TemplateContext {
    path: String,
    items: Vec<TemplateFile>
}

static DEFAULT_SIZE: &str = "-";
static DIRECTORY_FILE_TYPE: &str = "directory";
static DEFAULT_MIME_TYPE: &str = "application/unknown";

pub fn list(path: DirectoryPath, home: &str) -> Result<Template, Failure> {
    match readdir(path.as_ref(), home) {
        Ok(items) => {
            match relative_path(path.as_ref(), home) {
                Ok(path) => Ok(Template::render("index", &TemplateContext { path, items })),
                Err(_) => Err(Failure(Status::NotFound))
            }
        },
        // Redirect to 400 on readdir error
        Err(_reason) => Err(Failure(Status::BadRequest))
    }
}

fn relative_path(path: &Path, root: &str) -> Result<String, StripPrefixError> {
    match path.strip_prefix(root) {
        Ok(path) => Ok(String::from(Path::new("/").join(path).to_str().unwrap_or("/"))),
        Err(reason) => Err(reason)
    }
}

fn get_file_size(metadata: &Metadata) -> String {
    let size = metadata.len() as f64;
    return convert(size);
}

fn get_time(metadata: &Metadata) -> String {
    let date: DateTime<Utc> = metadata.modified().unwrap_or(SystemTime::now()).into();
    return date.format("%d/%m/%Y %Hh%M").to_string()
}

fn dir_entry_to_template_file(entry: DirEntry, home: &str) -> Option<TemplateFile> {
    let metadata = entry.metadata().unwrap();
    let path = entry.path();
    let file_type = metadata.file_type();
    let is_dir = file_type.is_dir();

    Some(TemplateFile {
        size:
            if is_dir {
                String::from(DEFAULT_SIZE)
            } else {
                get_file_size(&metadata)
            },
        name: String::from(path.file_name()?.to_str()?),
        is_symlink: file_type.is_symlink(),
        is_dir: is_dir,
        path: relative_path(path.as_ref(), home).unwrap(),
        mtime: get_time(&metadata),
        ext: String::from(path.extension().unwrap_or(OsStr::new("")).to_str()?),
        file_type:
            if is_dir {
                String::from(DIRECTORY_FILE_TYPE)
            } else {
                String::from(guess_mime_type_opt(&path).unwrap_or(DEFAULT_MIME_TYPE.parse().unwrap()).type_().as_str())
            }
    })
}

fn readdir(path: &Path, home: &str) -> Result<Vec<TemplateFile>, String> {
    match read_dir(path) {
        Ok(files) => {
            Ok(files.filter_map(|entry| {
                entry.ok().and_then(|e|
                    dir_entry_to_template_file(e, home)
                )
            }).collect())
        },
        Err(reason) => Err(reason.to_string())
    }
}
