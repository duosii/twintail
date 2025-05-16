use anstyle::{AnsiColor, Color};
use clap::builder::Styles;

// general colors
pub const TEXT: Color = Color::Ansi(AnsiColor::White);
pub const TEXT_VARIANT: Color = Color::Ansi(AnsiColor::BrightCyan);
pub const SUCCESS: Color = Color::Ansi(AnsiColor::BrightCyan);
pub const ERROR: Color = Color::Ansi(AnsiColor::Red);
pub const WARNING: Color = Color::Ansi(AnsiColor::BrightYellow);

const USAGE: Color = Color::Ansi(AnsiColor::BrightBlue);
const HEADER: Color = Color::Ansi(AnsiColor::BrightBlue);
const LITERAL: Color = Color::Ansi(AnsiColor::BrightCyan);
const INVALID: Color = Color::Ansi(AnsiColor::Red);
const VALID: Color = Color::Ansi(AnsiColor::BrightCyan);
const PLACEHOLDER: Color = Color::Ansi(AnsiColor::White);

pub fn get_clap_styles() -> Styles {
    Styles::styled()
        .usage(anstyle::Style::new().bold().fg_color(Some(USAGE)))
        .header(anstyle::Style::new().bold().fg_color(Some(HEADER)))
        .literal(anstyle::Style::new().fg_color(Some(LITERAL)))
        .invalid(anstyle::Style::new().bold().fg_color(Some(INVALID)))
        .error(anstyle::Style::new().bold().fg_color(Some(ERROR)))
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(VALID)),
        )
        .placeholder(anstyle::Style::new().fg_color(Some(PLACEHOLDER)))
}
