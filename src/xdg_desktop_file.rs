use crate::{xdg_desktop_value::*, xdg_parse_error::XdgParseError};
use lazy_static::lazy_static;
use onig::Regex;
use std::collections::HashMap;

type XdgDesktopSection = HashMap<String, XdgParseResult>;

#[derive(Debug)]
pub struct XdgDesktopFile {
    groups: HashMap<String, XdgDesktopSection>,
}

impl XdgDesktopFile {
    pub fn from_str(s: &str) -> Result<XdgDesktopFile, XdgParseError> {
        lazy_static! {
            static ref COMMENT_RE: Regex = Regex::new("#.*").unwrap();
            static ref SECTION_RE: Regex = Regex::new(r#"\[(.*)\]"#).unwrap();
        }
        let mut out = XdgDesktopFile { groups: HashMap::new() };
        let mut current_entry = HashMap::<String, XdgParseResult>::new();
        let mut current_entry_header: Option<&str> = None;
        for ln in s.lines() {
            match ln {
                comment if (COMMENT_RE.is_match(comment) | comment.trim().is_empty()) => {}
                section if SECTION_RE.is_match(section) => {
                    if current_entry_header.is_some() {
                        out.groups
                            .insert(current_entry_header.unwrap().to_string(), current_entry);
                        current_entry = HashMap::new();
                    }
                    current_entry_header = Some(section)
                }
                line => {
                    let (k, v) = XdgDesktopValue::from_kv(line);
                    current_entry.insert(k.to_string(), v);
                }
            }
        }
        if !current_entry.is_empty() {
            match current_entry_header {
                Some(s) => {
                    out.groups.insert(s.to_string(), current_entry);
                }
                None =>
                    return Err(XdgParseError::Other(
                        "File contains keys without section header",
                    )),
            }
        }
        Ok(out)
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn test_from_str() {
        let test_files = [
            "test/Alacritty.desktop",
            "test/htop.desktop",
            "test/org.pwmt.zathura.desktop",
        ];
        for f in &test_files {
            let contents = read_to_string(f).unwrap();
            assert!(XdgDesktopFile::from_str(&contents).is_ok())
        }
    }
}
