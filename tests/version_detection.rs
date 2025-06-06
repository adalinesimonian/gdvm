use gdvm::{
    i18n::I18n,
    project_version_detector::{detect_godot_version_in_path, find_project_file},
};
use tempfile::tempdir;

#[test]
fn test_find_project_file() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let proj = dir.path().join("project.godot");
    std::fs::write(&proj, "").unwrap();
    let found = find_project_file(&sub).unwrap();
    assert_eq!(found, proj);
}

#[test]
fn test_detect_godot_version_in_path() {
    let dir = tempdir().unwrap();
    let proj = dir.path().join("project.godot");
    std::fs::write(
        &proj,
        r#"
[application]
config/features=PackedStringArray("4.2")

[other]
foo=bar
"#,
    )
    .unwrap();
    let i18n = I18n::new(80).unwrap();
    let gv = detect_godot_version_in_path(&i18n, &proj).unwrap();
    assert_eq!(gv.major, Some(4));
    assert_eq!(gv.minor, Some(2));
}
