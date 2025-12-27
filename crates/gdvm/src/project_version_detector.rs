use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::eprintln_i18n;
use crate::i18n::I18n;
use crate::version_utils::GodotVersion;

/// Parsed representation of a `project.godot` file.
pub struct ParsedProject {
    config_version: Option<u32>,
    features_version: Option<String>,
    has_dotnet: bool,
}

impl ParsedProject {
    /// Parse raw `project.godot` contents.
    pub fn parse_str(contents: &str) -> Self {
        let config_version = parse_config_version(contents);
        // Check for [dotnet] section in project.godot
        let has_dotnet = contents.contains("[dotnet]");
        // Extract lines for `[application]` section.
        let features_version = extract_application_section(contents)
            .and_then(|lines| {
                // Look for `config/features` line and parse out version.
                lines
                    .iter()
                    .find(|line| line.trim_start().starts_with("config/features="))
                    .cloned()
            })
            .and_then(|line| parse_packed_string_array_for_version(&line));

        Self {
            config_version,
            features_version,
            has_dotnet,
        }
    }

    /// Convert the parsed fields into a detected `GodotVersion`.
    pub fn detected_version(&self) -> Option<GodotVersion> {
        // If the config_version is 4, then it's a Godot 3.x version.
        if self.config_version == Some(4) {
            return Some(GodotVersion {
                major: Some(3),
                minor: None,
                patch: None,
                subpatch: None,
                release_type: None,
                is_csharp: Some(self.has_dotnet),
            });
        }

        let version_candidate = self.features_version.as_ref()?;

        parse_version_string(version_candidate).map(|mut gv| {
            gv.is_csharp = Some(self.has_dotnet);
            gv
        })
    }
}

impl std::str::FromStr for ParsedProject {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse_str(s))
    }
}

/// IO helper that finds `project.godot`, reads it, and yields the parsed representation.
pub struct ProjectVersionProbe {
    contents: String,
}

impl ProjectVersionProbe {
    /// Walk upward from the provided path to locate `project.godot`, read its contents, and return
    /// a probe for further parsing.
    pub fn load<P: AsRef<Path>>(i18n: &I18n, path: P) -> Option<Self> {
        let project_file = find_project_file(path.as_ref())?;
        let contents = match fs::read_to_string(&project_file) {
            Ok(s) => s,
            Err(_) => {
                eprintln_i18n!(i18n, "error-failed-reading-project-godot");
                return None;
            }
        };

        Some(Self { contents })
    }

    /// Parse the loaded file contents into `ParsedProject` so version detection can run.
    pub fn parse(&self) -> ParsedProject {
        ParsedProject::parse_str(&self.contents)
    }
}

/// Detect the Godot version by looking for a `project.godot` file in the
/// directory tree that `path` belongs to, then parsing `[application]`
/// → `config/features` → `PackedStringArray(...)` for any `x.x` or `x.x.x`.
///
/// Returns `None` if the file cannot be found, parsed, or no version
/// is specified in `config/features`.
pub fn detect_godot_version_in_path<P: AsRef<Path>>(i18n: &I18n, path: P) -> Option<GodotVersion> {
    // Find the project root by walking up until we find `project.godot`.
    let probe = ProjectVersionProbe::load(i18n, path)?;

    // Parse the file, looking for the `[application]` section and
    //    `config/features=PackedStringArray(...)`.
    let parsed = probe.parse();
    parsed.detected_version()
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

    #[test]
    fn parsed_project_maps_config_version_four_to_godot_three() {
        let contents =
            "config_version=4\n[application]\nconfig/features=PackedStringArray(\"4.3\")\n";
        let parsed = super::ParsedProject::parse_str(contents);
        let detected = parsed.detected_version().unwrap();
        assert_eq!(detected.major, Some(3));
        assert_eq!(detected.is_csharp, Some(false));
    }

    #[test]
    fn parsed_project_reads_features_version_and_dotnet() {
        let contents = r#"
[dotnet]
[application]
config/features=PackedStringArray("4.3", "Forward Plus")
"#;
        let parsed = super::ParsedProject::parse_str(contents);
        let detected = parsed.detected_version().unwrap();
        assert_eq!(detected.major, Some(4));
        assert_eq!(detected.minor, Some(3));
        assert_eq!(detected.is_csharp, Some(true));
    }
}
