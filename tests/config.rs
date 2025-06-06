use gdvm::{config::Config, i18n::I18n};
use serial_test::serial;
use tempfile::tempdir;

#[test]
#[serial]
fn test_load_save_roundtrip() {
    let dir = tempdir().unwrap();
    unsafe {
        std::env::set_var("HOME", dir.path());
    }
    let i18n = I18n::new(80).unwrap();

    let cfg = Config {
        github_token: Some("token1".into()),
    };
    cfg.save(&i18n).unwrap();

    let mut loaded = Config::load(&i18n).unwrap();
    assert_eq!(loaded.github_token, Some("token1".to_string()));

    loaded.github_token = Some("token2".into());
    loaded.save(&i18n).unwrap();
    let loaded2 = Config::load(&i18n).unwrap();
    assert_eq!(loaded2.github_token, Some("token2".to_string()));
}
