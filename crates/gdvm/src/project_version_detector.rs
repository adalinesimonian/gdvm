use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::eprintln_i18n;
use crate::i18n::I18n;
use crate::version_utils::GodotVersion;

/// Detect the Godot version by looking for a `project.godot` file in the
/// directory tree that `path` belongs to, then parsing `[application]`
/// → `config/features` → `PackedStringArray(...)` for any `x.x` or `x.x.x`.
///
/// Returns `None` if the file cannot be found, parsed, or no version
/// is specified in `config/features`.
pub fn detect_godot_version_in_path<P: AsRef<Path>>(i18n: &I18n, path: P) -> Option<GodotVersion> {
    // Find the project root by walking up until we find `project.godot`.
    let project_file = find_project_file(path.as_ref())?;

    // Parse the file, looking for the `[application]` section and
    //    `config/features=PackedStringArray(...)`.
    let contents = match fs::read_to_string(&project_file) {
        Ok(s) => s,
        Err(_) => {
            eprintln_i18n!(i18n, "error-failed-reading-project-godot");
            return None;
        }
    };

    // Check for [dotnet] section in project.godot
    let is_csharp = contents.contains("[dotnet]");

    let config_version = parse_config_version(&contents);

    // If the config_version is 4, then it's a Godot 3.x version.
    if config_version == Some(4) {
        return Some(GodotVersion {
            major: Some(3),
            minor: None,
            patch: None,
            subpatch: None,
            release_type: None,
            is_csharp: Some(is_csharp),
        });
    }

    // Extract lines for `[application]` section.
    let application_lines = extract_application_section(&contents)?;

    // Look for `config/features` line and parse out version.
    let features_line = application_lines
        .iter()
        .find(|line| line.trim_start().starts_with("config/features="));

    let features_line = features_line?;

    // Expects something like: config/features=PackedStringArray("4.3", "Forward Plus")
    let version_candidate = parse_packed_string_array_for_version(features_line)?;

    // Parse the version string x.x or x.x.x into GodotVersion.
    match parse_version_string(&version_candidate) {
        Some(gv) => Some(GodotVersion {
            major: gv.major,
            minor: gv.minor,
            patch: gv.patch,
            subpatch: gv.subpatch,
            release_type: gv.release_type,
            is_csharp: Some(is_csharp),
        }),
        None => None,
    }
}

/// Walks up the directory tree starting from `start_path` until it finds
/// a file named `project.godot`. Returns the path to that file if found,
/// otherwise `None`.
pub fn find_project_file(start_path: &Path) -> Option<PathBuf> {
    let mut current = if start_path.is_file() {
        // If the file is "project.godot" itself, use that directly.
        if start_path.file_name() == Some(OsStr::new("project.godot")) {
            return Some(start_path.to_path_buf());
        }
        // Otherwise, work with the parent directory.
        start_path.parent().unwrap_or(start_path)
    } else {
        start_path
    };

    // Traverse up until we can't go further or we find `project.godot`.
    loop {
        let candidate = current.join("project.godot");
        if candidate.exists() {
            return Some(candidate);
        }

        match current.parent() {
            Some(parent) => {
                // Move one level up.
                current = parent;
            }
            None => {
                // Reached root without finding a `project.godot`.
                return None;
            }
        }
    }
}

/// Given the full contents of `project.godot`, extract just the lines in the
/// `[application]` section. Returns None if no `[application]` section is present.
fn extract_application_section(contents: &str) -> Option<Vec<String>> {
    let mut lines_in_application = Vec::new();
    let mut in_application_section = false;

    for line in contents.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // Encountered a new section.
            in_application_section = trimmed == "[application]";
            continue;
        }

        if in_application_section {
            // If we hit another section, stop collecting.
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                break;
            }
            lines_in_application.push(line.to_string());
        }
    }

    if lines_in_application.is_empty() {
        None
    } else {
        Some(lines_in_application)
    }
}

/// Looks at a line like `config/features=PackedStringArray("4.3", "Forward Plus")`
/// and returns the first substring that matches `x.x` or `x.x.x`.
fn parse_packed_string_array_for_version(line: &str) -> Option<String> {
    // Strip out the `config/features=` prefix.
    let eq_index = line.find('=')?;
    let value_part = line[(eq_index + 1)..].trim();

    // Expect something like `PackedStringArray("4.3", "Forward Plus")`.
    if !value_part.starts_with("PackedStringArray(") || !value_part.ends_with(')') {
        return None;
    }
    let inner_part = &value_part["PackedStringArray(".len()..value_part.len() - 1].trim();

    // Extract all quoted substrings. E.g. `"4.3", "Forward Plus"`
    let mut versions = Vec::new();
    let mut in_quotes = false;
    let mut prev_char_was_escape = false;
    let mut current_str = String::new();

    for c in inner_part.chars() {
        match c {
            '"' if !prev_char_was_escape => {
                in_quotes = !in_quotes;
                if !in_quotes {
                    versions.push(current_str.clone());
                }
                current_str.clear();
            }

            '\\' if in_quotes && !prev_char_was_escape => {
                prev_char_was_escape = true;
                current_str.push(c);
                continue;
            }

            _ if in_quotes => {
                current_str.push(c);
            }

            _ => {}
        }

        prev_char_was_escape = false;
    }

    // Find the first string that looks like a version `x.x` or `x.x.x`.
    versions.into_iter().find(|v| is_version_format(v))
}

/// Check if a string matches `x.x` or `x.x.x` where x are digits.
fn is_version_format(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }
    parts
        .iter()
        .all(|part| part.chars().all(|c| c.is_ascii_digit()))
}

/// Parse a version string `x.x` or `x.x.x` into a `GodotVersion`.
fn parse_version_string(version: &str) -> Option<GodotVersion> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }

    // Attempt to parse the first two parts. Return None if they fail to parse.
    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;

    let patch = if parts.len() == 3 {
        match parts[2].parse::<u32>() {
            Ok(p) => Some(p),
            Err(_) => return None,
        }
    } else {
        None
    };

    Some(GodotVersion {
        major: Some(major),
        minor: Some(minor),
        patch,
        subpatch: None,
        release_type: None,
        is_csharp: None,
    })
}

/// Parse the config version from the contents of `project.godot`.
fn parse_config_version(contents: &str) -> Option<u32> {
    for line in contents.lines() {
        if let Some(eq_index) = line.find('=') {
            let key = line[..eq_index].trim();
            let val = line[(eq_index + 1)..].trim();
            if key == "config_version" {
                return val.parse().ok();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_version_format() {
        assert!(super::is_version_format("4.1"));
        assert!(super::is_version_format("3.5.2"));
        assert!(!super::is_version_format("foo"));
        assert!(!super::is_version_format("1"));
    }

    #[test]
    fn test_parse_version_string() {
        let gv = super::parse_version_string("4.3").unwrap();
        assert_eq!(gv.major, Some(4));
        assert_eq!(gv.minor, Some(3));
        assert_eq!(gv.patch, None);
    }

    #[test]
    fn test_extract_application_section_and_parse() {
        let contents = r#"
[application]
config/features=PackedStringArray("4.3", "Forward Plus")

[other]
foo=bar
"#;
        let section = super::extract_application_section(contents).unwrap();
        assert!(section.iter().any(|l| l.contains("config/features")));
        let line = &section[0];
        let vers = super::parse_packed_string_array_for_version(line).unwrap();
        assert_eq!(vers, "4.3");
    }

    #[test]
    fn test_parse_config_version() {
        let contents = "config_version=4\n";
        assert_eq!(super::parse_config_version(contents), Some(4));
    }
}
