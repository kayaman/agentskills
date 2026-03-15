use std::path::Path;

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

/// Compute a deterministic SHA-256 hash of a directory's contents.
///
/// Files are sorted by relative path to ensure determinism across platforms.
/// Both file paths and contents are hashed. Dotfiles are excluded.
pub fn hash_directory(path: &Path) -> String {
    let mut hasher = Sha256::new();

    let mut files: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            !e.file_name()
                .to_string_lossy()
                .starts_with('.')
        })
        .map(|e| e.into_path())
        .collect();

    files.sort();

    for file_path in &files {
        if let Ok(rel) = file_path.strip_prefix(path) {
            hasher.update(rel.to_string_lossy().as_bytes());
            if let Ok(contents) = std::fs::read(file_path) {
                hasher.update(&contents);
            }
        }
    }

    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_sample_skill(dir: &Path) {
        fs::write(
            dir.join("SKILL.md"),
            "---\nname: sample\n---\n\n# Sample\n",
        )
        .unwrap();
        let refs = dir.join("references");
        fs::create_dir_all(&refs).unwrap();
        fs::write(refs.join("example.md"), "# Reference\n").unwrap();
    }

    #[test]
    fn test_deterministic() {
        let tmp = TempDir::new().unwrap();
        let skill = tmp.path().join("skill");
        fs::create_dir_all(&skill).unwrap();
        create_sample_skill(&skill);

        let h1 = hash_directory(&skill);
        let h2 = hash_directory(&skill);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sha256_length() {
        let tmp = TempDir::new().unwrap();
        let skill = tmp.path().join("skill");
        fs::create_dir_all(&skill).unwrap();
        create_sample_skill(&skill);

        let h = hash_directory(&skill);
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn test_changes_on_content_change() {
        let tmp = TempDir::new().unwrap();
        let skill = tmp.path().join("skill");
        fs::create_dir_all(&skill).unwrap();
        create_sample_skill(&skill);

        let h1 = hash_directory(&skill);
        fs::write(skill.join("SKILL.md"), "changed content").unwrap();
        let h2 = hash_directory(&skill);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_changes_on_new_file() {
        let tmp = TempDir::new().unwrap();
        let skill = tmp.path().join("skill");
        fs::create_dir_all(&skill).unwrap();
        create_sample_skill(&skill);

        let h1 = hash_directory(&skill);
        fs::write(skill.join("new-file.md"), "new content").unwrap();
        let h2 = hash_directory(&skill);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_ignores_dotfiles() {
        let tmp = TempDir::new().unwrap();
        let skill = tmp.path().join("skill");
        fs::create_dir_all(&skill).unwrap();
        create_sample_skill(&skill);

        let h1 = hash_directory(&skill);
        fs::write(skill.join(".gitignore"), "*.pyc").unwrap();
        let h2 = hash_directory(&skill);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let empty = tmp.path().join("empty");
        fs::create_dir_all(&empty).unwrap();

        let h = hash_directory(&empty);
        assert_eq!(h.len(), 64);
    }
}
