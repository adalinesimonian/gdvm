use gdvm::{godot_manager::find_godot_executable, i18n::I18n, zip_utils::extract_zip};
use std::io::Write;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;

#[test]
fn test_extract_zip_basic() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test.zip");
    {
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file("folder/file.txt", options).unwrap();
        zip.write_all(b"hello").unwrap();
        zip.finish().unwrap();
    }

    let out_dir = dir.path().join("out");
    let i18n = I18n::new(80).unwrap();
    extract_zip(&zip_path, &out_dir, &i18n).unwrap();
    let extracted = std::fs::read_to_string(out_dir.join("folder/file.txt")).unwrap();
    assert_eq!(extracted, "hello");
}

#[test]
fn test_find_godot_executable() {
    let dir = tempdir().unwrap();
    #[cfg(target_os = "windows")]
    let exe_path = dir.path().join("Godot_v4.0.exe");
    #[cfg(target_os = "linux")]
    let exe_path = dir.path().join("Godot_v4.0");
    #[cfg(target_os = "macos")]
    std::fs::create_dir_all(dir.path().join("Godot.app/Contents/MacOS")).unwrap();
    #[cfg(target_os = "macos")]
    let exe_path = dir.path().join("Godot.app/Contents/MacOS/Godot");
    std::fs::write(&exe_path, "").unwrap();
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&exe_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&exe_path, perms).unwrap();
    }
    let found = find_godot_executable(dir.path(), false).unwrap();
    assert!(found.is_some());
}
