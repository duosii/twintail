use crate::constants::color;
use clap::builder::Styles;

/// Get styles for ``clap``.
pub fn get_clap_styles() -> Styles {
    Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(color::clap::USAGE)),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(color::clap::HEADER)),
        )
        .literal(anstyle::Style::new().fg_color(Some(color::clap::LITERAL)))
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(color::clap::INVALID)),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(color::clap::ERROR)),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(color::clap::VALID)),
        )
        .placeholder(anstyle::Style::new().fg_color(Some(color::clap::PLACEHOLDER)))
}
