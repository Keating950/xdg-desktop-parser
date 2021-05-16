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

impl From<Vec<XdgDesktopValue>> for XdgDesktopValue {
    fn from(l: Vec<XdgDesktopValue>) -> Self {
        XdgDesktopValue::List(l)
    }
}

impl Into<String> for XdgDesktopValue {
    fn into(self) -> String {
        match self {
            XdgDesktopValue::IconString(s)
            | XdgDesktopValue::LocaleString(s)
            | XdgDesktopValue::String(s) => s.clone(), // I wish I didn't have to clone here
            XdgDesktopValue::Bool(b) => b.to_string(),
            XdgDesktopValue::Numeric(n) => n.to_string(),
            XdgDesktopValue::List(l) => {
                // Arbitrary size chosen
                let mut out = String::with_capacity(8 * l.len());
                for e in l.iter().map(XdgDesktopValue::to_string) {
                    out.push_str(&e);
                    out.push(';')
                }
                out
            }
        }
    }
}

impl std::fmt::Display for XdgDesktopValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

lazy_static! {
    static ref VAL_DELIMITER: Regex = Regex::new(r#"(?<!\\);"#).unwrap();
}

impl XdgDesktopValue {
    fn parse_string(s: &str) -> crate::Result<XdgDesktopValue> {
        Ok(XdgDesktopValue::String(s.to_string()))
    }

    fn parse_locale_string(s: &str) -> crate::Result<XdgDesktopValue> {
        Ok(XdgDesktopValue::LocaleString(s.to_string()))
    }

    fn parse_icon_string(s: &str) -> crate::Result<XdgDesktopValue> {
        Ok(XdgDesktopValue::IconString(s.to_string()))
    }

    fn parse_bool(s: &str) -> crate::Result<XdgDesktopValue> {
        Ok(s.parse::<bool>()?.into())
    }

    fn parse_numeric(s: &str) -> crate::Result<XdgDesktopValue> {
        Ok(s.parse::<f64>()?.into())
    }

    fn parse_plural(s: &str, f: fn(&str) -> crate::Result<XdgDesktopValue>) -> crate::Result<XdgDesktopValue> {
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

    fn try_types(s: &str) -> crate::Result<XdgDesktopValue> {
        const PARSE_FUNCS: [fn(&str) -> crate::Result<XdgDesktopValue>; 3] = [
            XdgDesktopValue::parse_bool,
            XdgDesktopValue::parse_numeric,
            XdgDesktopValue::parse_string,
        ];
        let mut parse_fn: Option<fn(&str) -> crate::Result<XdgDesktopValue>> = None;
        let mut out: Vec<XdgDesktopValue> = Vec::new();
        'outer: for v in VAL_DELIMITER.split(s) {
            match parse_fn {
                Some(f) => out.push(f(v)?),
                None => {
                    for f in &PARSE_FUNCS {
                        if let Ok(val) = f(s) {
                            out.push(val);
                            parse_fn = Some(*f);
                            continue 'outer;
                        }
                    }
                    // parse_string cannot fail.
                    unreachable!()
                }
            }
        }
        Ok(XdgDesktopValue::List(out))
    }

    pub fn from_kv(s: &str) -> (&str, crate::Result<XdgDesktopValue>) {
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

    #[test]
    fn test_list() {
        let input = "Keywords=system;process;task";
        assert!(XdgDesktopValue::from_kv(input).1.is_ok())
    }
}
