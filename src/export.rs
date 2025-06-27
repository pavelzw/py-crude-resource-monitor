use snafu::{Location, ResultExt, Snafu};
use std::path::Path;

mod firefox;
mod html;

#[derive(Debug, Snafu)]
pub enum ExportError {
    #[snafu(display("Error generating html report at {location}"))]
    Html {
        source: html::ExportError,
        #[snafu(implicit)]
        location: Location,
    },
    #[snafu(display("Error generating firefox report at {location}"))]
    Firefox {
        source: firefox::ExportError,
        #[snafu(implicit)]
        location: Location,
    },
}

/// Exports the profile data to a self-contained single-file HTML report.
pub fn export_html(data_dir: &Path, output_file: &Path) -> Result<(), ExportError> {
    html::export_report(data_dir, output_file).context(HtmlSnafu)
}

/// Exports the profile data to a Firefox-compatible JSON report.
pub fn export_firefox(data_dir: &Path, output_file: &Path) -> Result<(), ExportError> {
    firefox::export_report(data_dir, output_file).context(FirefoxSnafu)
}
