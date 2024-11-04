use anstyle::{AnsiColor, Color, RgbColor};

// colors for clap
pub mod clap {
    use anstyle::{AnsiColor, Color};
    pub const USAGE: Color = Color::Ansi(AnsiColor::BrightBlue);
    pub const HEADER: Color = Color::Ansi(AnsiColor::BrightBlue);
    pub const LITERAL: Color = Color::Ansi(AnsiColor::BrightCyan);
    pub const INVALID: Color = Color::Ansi(AnsiColor::Red);
    pub const ERROR: Color = Color::Ansi(AnsiColor::Red);
    pub const VALID: Color = Color::Ansi(AnsiColor::BrightCyan);
    pub const PLACEHOLDER: Color = Color::Ansi(AnsiColor::White);
}

// general colors
pub const TEXT: Color = Color::Ansi(AnsiColor::White);
pub const TEXT_VARIANT: Color = Color::Rgb(RgbColor(200, 200, 200));
pub const SUCCESS: Color = Color::Ansi(AnsiColor::BrightGreen);
