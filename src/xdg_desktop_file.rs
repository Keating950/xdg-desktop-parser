use crate::{xdg_desktop_value::*, xdg_parse_error::XdgParseError};
use lazy_static::lazy_static;
use onig::Regex;
use std::collections::HashMap;

type XdgDesktopSection = HashMap<String, crate::Result<XdgDesktopValue>>;

#[derive(Debug)]
pub struct XdgDesktopFile {
    sections: HashMap<String, XdgDesktopSection>,
}

impl XdgDesktopFile {
    pub fn from_str(s: &str) -> crate::Result<XdgDesktopFile> {
        lazy_static! {
            static ref COMMENT_RE: Regex = Regex::new("#.*").unwrap();
            static ref SECTION_RE: Regex = Regex::new(r#"\[(.*)\]"#).unwrap();
        }
        let mut out = XdgDesktopFile {
            sections: HashMap::new(),
        };
        let mut current_entry = HashMap::<String, crate::Result<XdgDesktopValue>>::new();
        let mut current_entry_header: Option<&str> = None;
        for ln in s.lines() {
            match ln {
                comment if (COMMENT_RE.is_match(comment) | comment.trim().is_empty()) => {}
                section if SECTION_RE.is_match(section) => {
                    if current_entry_header.is_some() {
                        out.sections
                            .insert(current_entry_header.unwrap().to_string(), current_entry);
                        current_entry = HashMap::new();
                    }
                    current_entry_header = Some(section)
                }
                line => {
                    if current_entry_header.is_none() {
                        return Err(XdgParseError::Other(
                            "File contains keys without section header",
                        ));
                    }
                    let (k, v) = XdgDesktopValue::from_kv(line);
                    current_entry.insert(k.to_string(), v);
                }
            }
        }
        if !current_entry.is_empty() {
            match current_entry_header {
                Some(s) => {
                    out.sections.insert(s.to_string(), current_entry);
                }
                None => {
                    return Err(XdgParseError::Other(
                        "File contains keys without section header",
                    ))
                }
            }
        }
        Ok(out)
    }

    pub fn sections(&self) -> impl Iterator<Item = (&str, &XdgDesktopSection)> {
        self.sections.iter().map(|(k, v)| (k.as_ref(), v))
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
            let parsed = XdgDesktopFile::from_str(&contents);
            assert!(parsed.is_ok());
            for grp in parsed.unwrap().sections() {
                for (_, v) in grp.1.iter() {
                    assert!(v.is_ok())
                }
            }
        }
    }
}
