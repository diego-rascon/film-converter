//! Finding the scans under whatever the user dropped, picked or named on the
//! command line.

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

/// Extensions we offer to import. Matches the Python file dialog's filter,
/// plus the other still formats the decoder already supports. The front end
/// asks for this list rather than keeping its own, so its file dialog can
/// never offer something the importer would then skip.
pub const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "tif", "tiff", "bmp", "webp"];

pub fn is_supported(path: &Path) -> bool {
    path.extension().and_then(OsStr::to_str).is_some_and(|ext| {
        SUPPORTED_EXTENSIONS
            .iter()
            .any(|known| known.eq_ignore_ascii_case(ext))
    })
}

/// The name a scan is listed under: its file name, or a placeholder for a
/// path that has none.
pub fn display_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || "unnamed".to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// A scan found on disk, with the size the walk already read.
#[derive(Debug)]
pub struct Found {
    pub path: PathBuf,
    pub bytes: u64,
}

/// Walks `roots`, expanding folders and everything under them, and returns
/// every supported image found, each once, ordered by path.
pub fn collect_images(roots: &[PathBuf]) -> Vec<Found> {
    let mut walk = Walk::default();
    for root in roots {
        walk.visit(root);
    }
    walk.found.sort_by(|a, b| a.path.cmp(&b.path));
    walk.found
}

#[derive(Default)]
struct Walk {
    found: Vec<Found>,
    /// Where each file really lives. The same file can be reached two ways —
    /// picked directly and again through its folder — so identity is the
    /// resolved path, not the spelling, and the first spelling is the one kept.
    seen: HashSet<PathBuf>,
    /// Folders already walked, by resolved path, so a link pointing back up
    /// the tree is followed once rather than for ever.
    walked: HashSet<PathBuf>,
}

impl Walk {
    /// Something named outright, or reached through a link: a folder is
    /// walked, a supported file is taken, anything else is skipped.
    fn visit(&mut self, path: &Path) {
        let Ok(metadata) = fs::metadata(path) else {
            return;
        };
        if metadata.is_dir() {
            self.walk_dir(path);
        } else if metadata.is_file() && is_supported(path) {
            let identity = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            self.take(path.to_path_buf(), identity, metadata.len());
        }
    }

    fn walk_dir(&mut self, dir: &Path) {
        let resolved = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        if !self.walked.insert(resolved.clone()) {
            return;
        }
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };

        // Read the whole folder first so recursion happens in a stable order.
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(fs::DirEntry::file_name);

        for entry in entries {
            let path = entry.path();
            // The entry's type comes with the listing, so telling files from
            // folders costs no extra call; only a link needs resolving.
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                self.walk_dir(&path);
            } else if kind.is_symlink() {
                self.visit(&path);
            } else if kind.is_file() && is_supported(&path) {
                // A plain file in a resolved folder is already resolved: no
                // need to ask the filesystem where it lives.
                let identity = resolved.join(entry.file_name());
                let bytes = entry.metadata().map_or(0, |m| m.len());
                self.take(path, identity, bytes);
            }
        }
    }

    fn take(&mut self, path: PathBuf, identity: PathBuf, bytes: u64) {
        if self.seen.insert(identity) {
            self.found.push(Found { path, bytes });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(found: &[Found]) -> Vec<String> {
        found
            .iter()
            .map(|f| f.path.file_name().unwrap().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn recognises_supported_extensions() {
        assert!(is_supported(Path::new("scan.JPG")));
        assert!(is_supported(Path::new("scan.tiff")));
        assert!(is_supported(Path::new("/a/b/scan.png")));
        assert!(!is_supported(Path::new("notes.txt")));
        assert!(!is_supported(Path::new("noextension")));
    }

    #[test]
    fn expands_folders_and_skips_other_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        fs::write(root.join("a.png"), b"12345").unwrap();
        fs::create_dir(root.join("sub")).unwrap();
        fs::write(root.join("sub").join("b.tif"), b"").unwrap();
        fs::write(root.join("notes.txt"), b"ignored").unwrap();

        let found = collect_images(std::slice::from_ref(&root));
        assert_eq!(
            names(&found),
            ["a.png", "b.tif"],
            "sub-folders included, text skipped"
        );
        assert_eq!(found[0].bytes, 5, "the size comes with the walk");

        // Naming a file directly and via its folder imports it once.
        let twice = collect_images(&[root.clone(), root.join("a.png")]);
        assert_eq!(twice.len(), 2, "no duplicate for the file named twice");
    }

    #[test]
    fn a_missing_path_finds_nothing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(collect_images(&[dir.path().join("gone")]).is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn a_link_back_up_the_tree_is_walked_once() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        fs::write(root.join("frame.jpg"), b"").unwrap();
        std::os::unix::fs::symlink(&root, root.join("loop")).unwrap();

        let found = collect_images(std::slice::from_ref(&root));
        assert_eq!(names(&found), ["frame.jpg"]);
    }

    #[cfg(unix)]
    #[test]
    fn a_linked_file_is_taken_once() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        fs::write(root.join("frame.jpg"), b"").unwrap();
        std::os::unix::fs::symlink(root.join("frame.jpg"), root.join("alias.jpg")).unwrap();

        // Both spellings resolve to one file; the first one met is kept.
        let found = collect_images(std::slice::from_ref(&root));
        assert_eq!(names(&found), ["alias.jpg"]);
    }
}
