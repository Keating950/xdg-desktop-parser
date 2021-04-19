use crate::xdg_parse_error::XdgParseError;
use lazy_static::lazy_static;
use onig::Regex;
use std::str;

#[derive(Debug, Clone)]
pub enum XdgDesktopValue {
    String(String),
    LocaleString(String),
    IconString(String),
    Bool(bool),
    Numeric(f64),
    List(Vec<XdgDesktopValue>),
}

pub type XdgParseResult = Result<XdgDesktopValue, XdgParseError>;

impl From<bool> for XdgDesktopValue {
    fn from(b: bool) -> Self {
        XdgDesktopValue::Bool(b)
    }
}

impl From<f64> for XdgDesktopValue {
    fn from(n: f64) -> Self {
        XdgDesktopValue::Numeric(n)
    }
}

lazy_static! {
    static ref VAL_DELIMITER: Regex = Regex::new(r#"(?<!\\);"#).unwrap();
}

impl XdgDesktopValue {
    fn parse_string(s: &str) -> XdgParseResult {
        Ok(XdgDesktopValue::String(s.to_string()))
    }

    fn parse_locale_string(s: &str) -> XdgParseResult {
        Ok(XdgDesktopValue::LocaleString(s.to_string()))
    }

    fn parse_icon_string(s: &str) -> XdgParseResult {
        Ok(XdgDesktopValue::IconString(s.to_string()))
    }

    fn parse_bool(s: &str) -> XdgParseResult {
        Ok(s.parse::<bool>()?.into())
    }

    fn parse_numeric(s: &str) -> XdgParseResult {
        Ok(s.parse::<f64>()?.into())
    }

    fn parse_plural(s: &str, f: fn(&str) -> XdgParseResult) -> XdgParseResult {
        let items: Result<Vec<XdgDesktopValue>, _> = VAL_DELIMITER.split(s).map(f).collect();
        Ok(XdgDesktopValue::List(items?))
    }

    fn strip_locale(s: &str) -> String {
        lazy_static! {
            static ref LOCALE_SUFFIX: Regex =
                Regex::new(r#"\[(?:[a-z]{2})(?:_[A-Z]{2})?(?:@\w+)?\]"#).unwrap();
        }
        LOCALE_SUFFIX.replace(s, "")
    }

    fn try_types(s: &str) -> XdgParseResult {
        let vals: Vec<&str> = VAL_DELIMITER.split(s).collect();
        let parse_funcs: [fn(&str) -> XdgParseResult; 3] = match vals.len() {
            1 => [
                XdgDesktopValue::parse_bool,
                XdgDesktopValue::parse_numeric,
                XdgDesktopValue::parse_string,
            ],
            _ => [
                |s| XdgDesktopValue::parse_plural(s, XdgDesktopValue::parse_bool),
                |s| XdgDesktopValue::parse_plural(s, XdgDesktopValue::parse_numeric),
                |s| XdgDesktopValue::parse_plural(s, XdgDesktopValue::parse_string),
            ],
        };
        for f in &parse_funcs {
            if let Ok(val) = f(s) {
                return Ok(val);
            }
        }
        // parse_string cannot fail
        unreachable!()
    }

    pub fn from_kv(s: &str) -> (&str, XdgParseResult) {
        let parse_strings =
            |s: &str| XdgDesktopValue::parse_plural(s, XdgDesktopValue::parse_string);
        let parse_locale_strings =
            |s: &str| XdgDesktopValue::parse_plural(s, XdgDesktopValue::parse_locale_string);
        let (k, v) = match s.split_once('=') {
            Some(tpl) => tpl,
            None => return (s, Err(XdgParseError::Other("No delimiter found in line"))),
        };
        let key_base = XdgDesktopValue::strip_locale(k);
        #[rustfmt::skip]
            let parse_fn = match key_base.as_ref() {
            "Type"
            | "Version"
            | "Exec"
            | "TryExec"
            | "Path"
            | "StartupWMClass"
            | "URL" => XdgDesktopValue::parse_string,
            "Name" | "GenericName" | "Comment" => XdgDesktopValue::parse_locale_string,
            "NoDisplay"
            | "Hidden"
            | "Terminal"
            | "StartupNotify"
            | "PrefersNonDefaultGPU"
            | "DBusActivatable" => XdgDesktopValue::parse_bool,
            "Icon" => XdgDesktopValue::parse_icon_string,
            "Keywords" => parse_locale_strings,
            "OnlyShowIn"
            | "NotShowIn"
            | "Actions"
            | "MimeType"
            | "Categories"
            | "Implements" => parse_strings,
            _ => XdgDesktopValue::try_types
        };
        match parse_fn(v) {
            Ok(xdg) => (k, Ok(xdg)),
            Err(e) => (k, Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localize_strip() {
        let items = ["Name", "Name[es]", "Name[es_CL]", "Name[sr@Latn]"];
        for i in &items {
            assert_eq!("Name", XdgDesktopValue::strip_locale(i), "\nInput: {}\n", i);
        }
    }
}
