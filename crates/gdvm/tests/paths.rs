#![cfg(feature = "integration-tests")]

use gdvm::{
    config::{Config, ConfigOps, get_absolute_path},
    i18n::I18n,
    paths::GdvmPaths,
};
use serial_test::serial;
use std::path::Path;
use tempfile::tempdir;

fn with_test_home<F, R>(path: &Path, f: F) -> R
where
    F: FnOnce() -> R,
{
    let previous = std::env::var("GDVM_TEST_HOME").ok();

    unsafe {
        std::env::set_var("GDVM_TEST_HOME", path);
    }

    let result = f();

    if let Some(val) = previous {
        unsafe {
            std::env::set_var("GDVM_TEST_HOME", val);
        }
    } else {
        unsafe {
            std::env::remove_var("GDVM_TEST_HOME");
        }
    }

    result
}

#[test]
fn test_get_absolute_path_normalizes_relative_paths() {
    let path = get_absolute_path("subdir/../installs").unwrap();

    assert!(path.is_absolute());
    assert!(
        !path
            .components()
            .any(|component| component == std::path::Component::ParentDir)
    );
    assert_eq!(path, std::env::current_dir().unwrap().join("installs"));
}

#[test]
fn test_get_absolute_path_rejects_empty_strings() {
    assert!(get_absolute_path("").is_err());
    assert!(get_absolute_path("   ").is_err());
}

#[test]
fn test_get_absolute_path_rejects_existing_files() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("not_a_dir.txt");
    std::fs::write(&file_path, "data").unwrap();

    assert!(get_absolute_path(file_path.to_string_lossy().as_ref()).is_err());
}

#[test]
#[serial]
fn test_gdvm_paths_uses_normalized_absolute_paths() {
    let dir = tempdir().unwrap();
    let i18n = I18n::new().unwrap();

    with_test_home(dir.path(), || {
        let mut cfg = Config::default();
        cfg.set_value("install.path", "./custom-installs").unwrap();
        cfg.set_value("cache.path", "./custom-cache").unwrap();
        cfg.save(&i18n).unwrap();

        let paths = GdvmPaths::new(&i18n).unwrap();

        assert!(paths.installs().is_absolute());
        assert!(paths.cache_dir().is_absolute());
        assert!(
            !paths
                .installs()
                .components()
                .any(|component| component == std::path::Component::ParentDir)
        );
        assert!(
            !paths
                .cache_dir()
                .components()
                .any(|component| component == std::path::Component::ParentDir)
        );
    });
}
