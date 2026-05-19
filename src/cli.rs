use clap::{Parser, ValueEnum};
use rustyline::{ColorMode as RlColorMode, EditMode as RlEditMode};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The file to run. If not provided, starts the REPL.
    pub file: Option<PathBuf>,

    #[command(flatten)]
    pub repl: ReplConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, bon::Builder, clap::Args)]
pub struct ReplConfig {
    #[arg(long, value_enum, default_value_t = EditMode::Vi)]
    #[builder(default = EditMode::Vi)]
    pub edit_mode: EditMode,

    #[arg(long, value_enum, default_value_t = ColorMode::Enabled)]
    #[builder(default = ColorMode::Enabled)]
    pub color_mode: ColorMode,

    #[arg(long, default_value = ".ff_history")]
    #[builder(default = ".ff_history", into)]
    pub history_path: PathBuf,

    #[arg(long, default_value_t = false)]
    #[builder(default = false)]
    pub should_write_history: bool,
}

impl Default for ReplConfig {
    fn default() -> Self {
        ReplConfig::builder().build()
    }
}

/// Owned version of rustyline::EditMode so we can derive clap::ValueEnum
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum EditMode {
    Emacs,
    Vi,
}

impl From<EditMode> for RlEditMode {
    fn from(value: EditMode) -> Self {
        match value {
            EditMode::Emacs => RlEditMode::Emacs,
            EditMode::Vi => RlEditMode::Vi,
        }
    }
}

/// Owned version of rustyline::ColorMode so we can derive clap::ValueEnum
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorMode {
    Enabled,
    Disabled,
    Force,
}

impl From<ColorMode> for RlColorMode {
    fn from(value: ColorMode) -> Self {
        match value {
            ColorMode::Enabled => RlColorMode::Enabled,
            ColorMode::Disabled => RlColorMode::Disabled,
            ColorMode::Force => RlColorMode::Forced,
        }
    }
}
