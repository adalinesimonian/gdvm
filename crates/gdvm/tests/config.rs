#![cfg(feature = "integration-tests")]

use gdvm::{config::Config, i18n::I18n};
use serial_test::serial;
use tempfile::tempdir;

fn with_test_home<F, R>(path: &std::path::Path, f: F) -> R
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
#[serial]
fn test_load_save_roundtrip() {
    let dir = tempdir().unwrap();
    let i18n = I18n::new(80).unwrap();

    with_test_home(dir.path(), || {
        let cfg = Config {
            github_token: Some("token1".into()),
            global_installs_location: None,
            global_launch_shortcut: None,
        };
        cfg.save(&i18n).unwrap();
    });

    let loaded = with_test_home(dir.path(), || Config::load(&i18n).unwrap());
    assert_eq!(loaded.github_token, Some("token1".to_string()));

    with_test_home(dir.path(), || {
        let cfg = Config {
            github_token: Some("token2".into()),
            global_installs_location: None,
            global_launch_shortcut: None,
        };
        cfg.save(&i18n).unwrap();
    });

    let loaded2 = with_test_home(dir.path(), || Config::load(&i18n).unwrap());
    assert_eq!(loaded2.github_token, Some("token2".to_string()));
}

#[test]
#[serial]
fn test_change_installs_location_config() {
    let dir = tempdir().unwrap();
    let i18n = I18n::new(80).unwrap();

    with_test_home(dir.path(), || {
        let cfg = Config {
            github_token: None,
            global_installs_location: Some(dir.path().join("test_installs")),
            global_launch_shortcut: None,
        };
        cfg.save(&i18n).unwrap();
    });

    let loaded = with_test_home(dir.path(), || Config::load(&i18n).unwrap());
    assert_eq!(
        loaded.global_installs_location,
        Some(dir.path().join("test_installs"))
    );
}

#[test]
#[serial]
fn test_change_launch_shortcut_config() {
    let dir = tempdir().unwrap();
    let i18n = I18n::new(80).unwrap();

    with_test_home(dir.path(), || {
        let cfg = Config {
            github_token: None,
            global_installs_location: None,
            global_launch_shortcut: Some(true),
        };
        cfg.save(&i18n).unwrap();
    });

    let loaded = with_test_home(dir.path(), || Config::load(&i18n).unwrap());
    assert_eq!(loaded.global_launch_shortcut, Some(true));
}
