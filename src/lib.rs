mod xdg_desktop_file;
mod xdg_desktop_value;
mod xdg_parse_error;

pub type Result<T> = std::result::Result<T, XdgParseError>;
pub use xdg_desktop_file::XdgDesktopFile;
pub use xdg_desktop_value::XdgDesktopValue;
pub use xdg_parse_error::XdgParseError;
