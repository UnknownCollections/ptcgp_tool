#![allow(dead_code)]

use cursive::style::{BaseColor::*, Color, Effect, Style};
use cursive::theme::{BorderStyle, Palette, PaletteColor, PaletteStyle, Theme};
use cursive::With;

/// Constructs a new theme with a retro palette that is modified by the provided function.
///
/// # Arguments
///
/// * `shadow` - Enables or disables a shadow effect for the theme.
/// * `borders` - Specifies the border style to be used.
/// * `palette_modifier` - A function that applies additional modifications to a retro palette.
///
/// # Returns
///
/// A `Theme` instance configured with the given properties.
fn build_theme(
    shadow: bool,
    borders: BorderStyle,
    palette_modifier: impl FnOnce(&mut Palette),
) -> Theme {
    Theme {
        shadow,
        borders,
        palette: Palette::retro().with(palette_modifier),
    }
}

/// Returns a "Midnight" theme for the application.
///
/// This theme uses a dark color scheme with deep blues and grays to evoke a midnight ambiance.
pub fn midnight_theme() -> Theme {
    build_theme(true, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(8, 10, 14);
        palette[PaletteColor::Shadow] = Color::Rgb(20, 20, 28);
        palette[PaletteColor::View] = Color::Rgb(8, 10, 14);
        palette[PaletteColor::Primary] = Color::Rgb(200, 200, 220);
        palette[PaletteColor::Secondary] = Color::Rgb(128, 144, 160);
        palette[PaletteColor::Tertiary] = Color::Rgb(100, 112, 130);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(120, 160, 255);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(80, 120, 200);
        palette[PaletteColor::Highlight] = Color::Rgb(42, 130, 228);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(42, 70, 100);
        palette[PaletteColor::HighlightText] = Color::Rgb(255, 255, 255);
        palette[PaletteStyle::Background] = Style::none();
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Color::Rgb(127, 127, 127));
        palette[PaletteStyle::TitlePrimary] = Style::from(Blue.light()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Blue.dark());
        palette[PaletteStyle::Highlight] = Style::from(Blue.light()).combine(Effect::Bold);
        palette[PaletteStyle::HighlightInactive] = Style::from(Blue.dark());
        palette[PaletteStyle::Shadow] = Style::from(Black.dark());
    })
}

/// Returns a "Nord" theme for the application.
///
/// This theme is inspired by the Nord color palette, featuring cool blues and muted tones.
pub fn nord_theme() -> Theme {
    build_theme(false, BorderStyle::Outset, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(46, 52, 64);
        palette[PaletteColor::Shadow] = Color::Rgb(59, 66, 82);
        palette[PaletteColor::View] = Color::Rgb(46, 52, 64);
        palette[PaletteColor::Primary] = Color::Rgb(216, 222, 233);
        palette[PaletteColor::Secondary] = Color::Rgb(143, 188, 187);
        palette[PaletteColor::Tertiary] = Color::Rgb(180, 142, 173);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(136, 192, 208);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(163, 190, 140);
        palette[PaletteColor::Highlight] = Color::Rgb(129, 161, 193);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(59, 66, 82);
        palette[PaletteColor::HighlightText] = Color::Rgb(236, 239, 244);
        palette[PaletteStyle::Background] = Style::from(Black.dark());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Cyan.light());
        palette[PaletteStyle::TitlePrimary] = Style::from(Cyan.light()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Green.dark());
        palette[PaletteStyle::Highlight] = Style::from(Cyan.dark()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Cyan.dark()).combine(Effect::Italic);
        palette[PaletteStyle::Shadow] = Style::none();
    })
}

/// Returns a "Dracula" theme for the application.
///
/// This theme features a dark color scheme with vivid accent colors, reminiscent of the Dracula style.
pub fn dracula_theme() -> Theme {
    build_theme(true, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(40, 42, 54);
        palette[PaletteColor::Shadow] = Color::Rgb(30, 32, 42);
        palette[PaletteColor::View] = Color::Rgb(40, 42, 54);
        palette[PaletteColor::Primary] = Color::Rgb(248, 248, 242);
        palette[PaletteColor::Secondary] = Color::Rgb(189, 147, 249);
        palette[PaletteColor::Tertiary] = Color::Rgb(98, 114, 164);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(255, 121, 198);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(189, 147, 249);
        palette[PaletteColor::Highlight] = Color::Rgb(80, 250, 123);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(68, 71, 90);
        palette[PaletteColor::HighlightText] = Color::Rgb(40, 42, 54);
        palette[PaletteStyle::Background] = Style::from(Black.dark());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Magenta.light());
        palette[PaletteStyle::TitlePrimary] = Style::from(Magenta.dark()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Magenta.light());
        palette[PaletteStyle::Highlight] = Style::from(Green.light()).combine(Effect::Bold);
        palette[PaletteStyle::HighlightInactive] = Style::from(Green.dark());
        palette[PaletteStyle::Shadow] = Style::none();
    })
}

/// Returns a "Solarized Dark" theme for the application.
///
/// This theme applies the Solarized Dark color palette with a mix of cool and warm tones.
pub fn solarized_dark_theme() -> Theme {
    build_theme(true, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(0, 43, 54);
        palette[PaletteColor::Shadow] = Color::Rgb(7, 54, 66);
        palette[PaletteColor::View] = Color::Rgb(0, 43, 54);
        palette[PaletteColor::Primary] = Color::Rgb(131, 148, 150);
        palette[PaletteColor::Secondary] = Color::Rgb(88, 110, 117);
        palette[PaletteColor::Tertiary] = Color::Rgb(101, 123, 131);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(181, 137, 0);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(203, 75, 22);
        palette[PaletteColor::Highlight] = Color::Rgb(0, 139, 139);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(0, 106, 106);
        palette[PaletteColor::HighlightText] = Color::Rgb(253, 246, 227);
        palette[PaletteStyle::Background] = Style::none();
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(Color::Rgb(127, 127, 127));
        palette[PaletteStyle::Secondary] = Style::from(Cyan.dark());
        palette[PaletteStyle::TitlePrimary] = Style::from(Yellow.dark()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Red.dark());
        palette[PaletteStyle::Highlight] = Style::from(Cyan.light()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Cyan.dark());
        palette[PaletteStyle::Shadow] = Style::from(Black.dark());
    })
}

/// Returns a "Gruvbox Dark" theme for the application.
///
/// This theme uses the Gruvbox color palette with warm, earthy tones for a retro look.
pub fn gruvbox_dark_theme() -> Theme {
    build_theme(false, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(40, 40, 40);
        palette[PaletteColor::Shadow] = Color::Rgb(50, 48, 47);
        palette[PaletteColor::View] = Color::Rgb(40, 40, 40);
        palette[PaletteColor::Primary] = Color::Rgb(235, 219, 178);
        palette[PaletteColor::Secondary] = Color::Rgb(168, 153, 132);
        palette[PaletteColor::Tertiary] = Color::Rgb(213, 196, 161);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(251, 73, 52);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(184, 187, 38);
        palette[PaletteColor::Highlight] = Color::Rgb(250, 189, 47);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(146, 131, 116);
        palette[PaletteColor::HighlightText] = Color::Rgb(40, 40, 40);
        palette[PaletteStyle::Background] = Style::from(Black.dark());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Yellow.dark());
        palette[PaletteStyle::TitlePrimary] = Style::from(Red.light()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Green.dark());
        palette[PaletteStyle::Highlight] = Style::from(Yellow.light()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Color::Rgb(127, 127, 127));
        palette[PaletteStyle::Shadow] = Style::none();
    })
}

/// Returns a "Monokai" theme for the application.
///
/// This theme is inspired by the popular Monokai color scheme, featuring high contrast colors.
pub fn monokai_theme() -> Theme {
    build_theme(true, BorderStyle::None, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(39, 40, 34);
        palette[PaletteColor::Shadow] = Color::Rgb(29, 30, 24);
        palette[PaletteColor::View] = Color::Rgb(39, 40, 34);
        palette[PaletteColor::Primary] = Color::Rgb(248, 248, 242);
        palette[PaletteColor::Secondary] = Color::Rgb(166, 226, 46);
        palette[PaletteColor::Tertiary] = Color::Rgb(102, 217, 239);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(249, 38, 114);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(174, 129, 255);
        palette[PaletteColor::Highlight] = Color::Rgb(253, 151, 31);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(133, 125, 97);
        palette[PaletteColor::HighlightText] = Color::Rgb(39, 40, 34);
        palette[PaletteStyle::Background] = Style::none();
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Green.light());
        palette[PaletteStyle::TitlePrimary] = Style::from(Red.dark()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Magenta.light());
        palette[PaletteStyle::Highlight] = Style::from(Yellow.dark()).combine(Effect::Bold);
        palette[PaletteStyle::HighlightInactive] = Style::from(Yellow.dark());
        palette[PaletteStyle::Shadow] = Style::from(Black.dark());
    })
}

/// Returns an "Oceanic Dark" theme for the application.
///
/// This theme features a dark background with ocean-inspired blue and cyan accents.
pub fn oceanic_dark_theme() -> Theme {
    build_theme(true, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(34, 40, 49);
        palette[PaletteColor::Shadow] = Color::Rgb(24, 30, 39);
        palette[PaletteColor::View] = Color::Rgb(34, 40, 49);
        palette[PaletteColor::Primary] = Color::Rgb(171, 178, 191);
        palette[PaletteColor::Secondary] = Color::Rgb(88, 155, 200);
        palette[PaletteColor::Tertiary] = Color::Rgb(52, 101, 164);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(38, 139, 210);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(108, 113, 196);
        palette[PaletteColor::Highlight] = Color::Rgb(42, 161, 152);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(59, 66, 82);
        palette[PaletteColor::HighlightText] = Color::Rgb(238, 232, 213);
        palette[PaletteStyle::Background] = Style::from(Black.dark());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Blue.dark());
        palette[PaletteStyle::TitlePrimary] = Style::from(Blue.light()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Blue.dark()).combine(Effect::Italic);
        palette[PaletteStyle::Highlight] = Style::from(Cyan.light()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Cyan.dark());
        palette[PaletteStyle::Shadow] = Style::none();
    })
}

/// Returns a "One Dark" theme for the application.
///
/// This theme is based on the One Dark color palette, offering a modern dark look.
pub fn one_dark_theme() -> Theme {
    build_theme(true, BorderStyle::Outset, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(40, 44, 52);
        palette[PaletteColor::Shadow] = Color::Rgb(30, 33, 39);
        palette[PaletteColor::View] = Color::Rgb(40, 44, 52);
        palette[PaletteColor::Primary] = Color::Rgb(171, 178, 191);
        palette[PaletteColor::Secondary] = Color::Rgb(198, 120, 221);
        palette[PaletteColor::Tertiary] = Color::Rgb(224, 108, 117);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(97, 175, 239);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(152, 195, 121);
        palette[PaletteColor::Highlight] = Color::Rgb(229, 192, 123);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(67, 72, 80);
        palette[PaletteColor::HighlightText] = Color::Rgb(40, 44, 52);
        palette[PaletteStyle::Background] = Style::none();
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Magenta.light());
        palette[PaletteStyle::TitlePrimary] = Style::from(Blue.light()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Green.light());
        palette[PaletteStyle::Highlight] = Style::from(Yellow.dark()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Color::Rgb(127, 127, 127));
        palette[PaletteStyle::Shadow] = Style::from(Black.dark());
    })
}

/// Returns a "Dark Forest" theme for the application.
///
/// This theme uses natural, earthy tones to evoke the feeling of a dark forest.
pub fn dark_forest_theme() -> Theme {
    build_theme(false, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(25, 34, 31);
        palette[PaletteColor::Shadow] = Color::Rgb(15, 24, 21);
        palette[PaletteColor::View] = Color::Rgb(25, 34, 31);
        palette[PaletteColor::Primary] = Color::Rgb(191, 220, 195);
        palette[PaletteColor::Secondary] = Color::Rgb(131, 160, 135);
        palette[PaletteColor::Tertiary] = Color::Rgb(80, 125, 90);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(0, 145, 85);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(0, 105, 65);
        palette[PaletteColor::Highlight] = Color::Rgb(0, 165, 95);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(0, 125, 70);
        palette[PaletteColor::HighlightText] = Color::Rgb(224, 255, 235);
        palette[PaletteStyle::Background] = Style::from(Green.dark());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Green.light());
        palette[PaletteStyle::TitlePrimary] = Style::from(Green.dark()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Green.dark()).combine(Effect::Italic);
        palette[PaletteStyle::Highlight] = Style::from(Green.light()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Green.dark());
        palette[PaletteStyle::Shadow] = Style::none();
    })
}

/// Returns a "Molokai Dark" theme for the application.
///
/// This theme is inspired by the Molokai color scheme, featuring strong contrasts and vivid colors.
pub fn molokai_dark_theme() -> Theme {
    build_theme(true, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(25, 25, 25);
        palette[PaletteColor::Shadow] = Color::Rgb(15, 15, 15);
        palette[PaletteColor::View] = Color::Rgb(25, 25, 25);
        palette[PaletteColor::Primary] = Color::Rgb(205, 205, 192);
        palette[PaletteColor::Secondary] = Color::Rgb(249, 38, 114);
        palette[PaletteColor::Tertiary] = Color::Rgb(166, 226, 46);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(174, 129, 255);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(82, 171, 218);
        palette[PaletteColor::Highlight] = Color::Rgb(253, 151, 31);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(132, 99, 10);
        palette[PaletteColor::HighlightText] = Color::Rgb(40, 40, 40);
        palette[PaletteStyle::Background] = Style::none();
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Red.light());
        palette[PaletteStyle::TitlePrimary] = Style::from(Magenta.dark()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Cyan.dark());
        palette[PaletteStyle::Highlight] = Style::from(Yellow.dark()).combine(Effect::Bold);
        palette[PaletteStyle::HighlightInactive] = Style::from(Yellow.dark());
        palette[PaletteStyle::Shadow] = Style::from(Black.dark());
    })
}

/// Returns a "Solarized Light" theme for the application.
///
/// This theme uses the Solarized Light palette for a bright, easy-on-the-eyes interface.
pub fn solarized_light_theme() -> Theme {
    build_theme(false, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(253, 246, 227);
        palette[PaletteColor::Shadow] = Color::Rgb(238, 232, 213);
        palette[PaletteColor::View] = Color::Rgb(253, 246, 227);
        palette[PaletteColor::Primary] = Color::Rgb(101, 123, 131);
        palette[PaletteColor::Secondary] = Color::Rgb(88, 110, 117);
        palette[PaletteColor::Tertiary] = Color::Rgb(147, 161, 161);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(181, 137, 0);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(211, 54, 130);
        palette[PaletteColor::Highlight] = Color::Rgb(42, 161, 152);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(147, 161, 161);
        palette[PaletteColor::HighlightText] = Color::Rgb(253, 246, 227);
        palette[PaletteStyle::Background] = Style::none();
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(Color::Rgb(80, 80, 80));
        palette[PaletteStyle::Secondary] = Style::from(Cyan.dark());
        palette[PaletteStyle::TitlePrimary] = Style::from(Yellow.dark()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Magenta.dark());
        palette[PaletteStyle::Highlight] = Style::from(Cyan.light()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Cyan.dark()).combine(Effect::Italic);
        palette[PaletteStyle::Shadow] = Style::none();
    })
}

/// Returns a "Gruvbox Light" theme for the application.
///
/// This theme applies the Gruvbox palette with lighter tones for a bright interface.
pub fn gruvbox_light_theme() -> Theme {
    build_theme(false, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(251, 241, 199);
        palette[PaletteColor::Shadow] = Color::Rgb(235, 219, 178);
        palette[PaletteColor::View] = Color::Rgb(251, 241, 199);
        palette[PaletteColor::Primary] = Color::Rgb(60, 56, 54);
        palette[PaletteColor::Secondary] = Color::Rgb(124, 111, 100);
        palette[PaletteColor::Tertiary] = Color::Rgb(80, 73, 69);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(215, 153, 33);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(143, 63, 113);
        palette[PaletteColor::Highlight] = Color::Rgb(69, 133, 136);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(189, 174, 147);
        palette[PaletteColor::HighlightText] = Color::Rgb(251, 241, 199);
        palette[PaletteStyle::Background] = Style::none();
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(Black.dark());
        palette[PaletteStyle::Secondary] = Style::from(Yellow.dark());
        palette[PaletteStyle::TitlePrimary] = Style::from(Yellow.light()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Magenta.dark());
        palette[PaletteStyle::Highlight] = Style::from(Cyan.dark()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Color::Rgb(127, 127, 127));
        palette[PaletteStyle::Shadow] = Style::from(Black.light());
    })
}

/// Returns a "GitHub Light" theme for the application.
///
/// This theme is designed to mimic the light color scheme used on GitHub.
pub fn github_light_theme() -> Theme {
    build_theme(false, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(255, 255, 255);
        palette[PaletteColor::Shadow] = Color::Rgb(230, 230, 230);
        palette[PaletteColor::View] = Color::Rgb(255, 255, 255);
        palette[PaletteColor::Primary] = Color::Rgb(36, 41, 46);
        palette[PaletteColor::Secondary] = Color::Rgb(87, 96, 106);
        palette[PaletteColor::Tertiary] = Color::Rgb(110, 119, 129);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(3, 102, 214);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(34, 134, 58);
        palette[PaletteColor::Highlight] = Color::Rgb(200, 241, 200);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(230, 230, 230);
        palette[PaletteColor::HighlightText] = Color::Rgb(36, 41, 46);
        palette[PaletteStyle::Background] = Style::from(White.light());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(Black.dark());
        palette[PaletteStyle::Secondary] = Style::from(Black.light());
        palette[PaletteStyle::TitlePrimary] = Style::from(Blue.light()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Green.light());
        palette[PaletteStyle::Highlight] = Style::from(Green.dark()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Black.light());
        palette[PaletteStyle::Shadow] = Style::none();
    })
}

/// Returns a "Nord Light" theme for the application.
///
/// This theme applies the Nord palette with a light background and cool accent colors.
pub fn nord_light_theme() -> Theme {
    build_theme(false, BorderStyle::Outset, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(236, 239, 244);
        palette[PaletteColor::Shadow] = Color::Rgb(216, 222, 233);
        palette[PaletteColor::View] = Color::Rgb(236, 239, 244);
        palette[PaletteColor::Primary] = Color::Rgb(76, 86, 106);
        palette[PaletteColor::Secondary] = Color::Rgb(67, 76, 94);
        palette[PaletteColor::Tertiary] = Color::Rgb(59, 66, 82);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(94, 129, 172);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(143, 188, 187);
        palette[PaletteColor::Highlight] = Color::Rgb(136, 192, 208);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(203, 214, 226);
        palette[PaletteColor::HighlightText] = Color::Rgb(46, 52, 64);
        palette[PaletteStyle::Background] = Style::from(White.light());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(Black.dark());
        palette[PaletteStyle::Secondary] = Style::from(Blue.dark());
        palette[PaletteStyle::TitlePrimary] = Style::from(Blue.light()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Cyan.light());
        palette[PaletteStyle::Highlight] = Style::from(Cyan.dark()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(White.dark());
        palette[PaletteStyle::Shadow] = Style::none();
    })
}

/// Returns a "Papercolor Light" theme for the application.
///
/// This theme provides a bright, minimalistic color scheme reminiscent of paper.
pub fn papercolor_light_theme() -> Theme {
    build_theme(false, BorderStyle::None, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(255, 255, 255);
        palette[PaletteColor::Shadow] = Color::Rgb(222, 222, 222);
        palette[PaletteColor::View] = Color::Rgb(255, 255, 255);
        palette[PaletteColor::Primary] = Color::Rgb(68, 68, 68);
        palette[PaletteColor::Secondary] = Color::Rgb(102, 102, 102);
        palette[PaletteColor::Tertiary] = Color::Rgb(136, 136, 136);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(0, 95, 135);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(175, 135, 0);
        palette[PaletteColor::Highlight] = Color::Rgb(95, 175, 215);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(175, 175, 175);
        palette[PaletteColor::HighlightText] = Color::Rgb(255, 255, 255);
        palette[PaletteStyle::Background] = Style::from(White.light());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(Black.dark());
        palette[PaletteStyle::Secondary] = Style::from(Black.light());
        palette[PaletteStyle::TitlePrimary] = Style::from(Blue.dark()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Yellow.dark()).combine(Effect::Italic);
        palette[PaletteStyle::Highlight] = Style::from(Blue.light()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(White.dark());
        palette[PaletteStyle::Shadow] = Style::from(Color::Rgb(127, 127, 127));
    })
}

/// Returns a "Tokyo Night Storm" theme for the application.
///
/// This theme uses dark tones with vibrant accents inspired by the Tokyo Night color scheme.
pub fn tokyo_night_storm_theme() -> Theme {
    build_theme(true, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(36, 40, 59);
        palette[PaletteColor::Shadow] = Color::Rgb(27, 31, 48);
        palette[PaletteColor::View] = Color::Rgb(36, 40, 59);
        palette[PaletteColor::Primary] = Color::Rgb(192, 202, 245);
        palette[PaletteColor::Secondary] = Color::Rgb(122, 162, 247);
        palette[PaletteColor::Tertiary] = Color::Rgb(86, 95, 137);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(157, 124, 216);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(187, 154, 247);
        palette[PaletteColor::Highlight] = Color::Rgb(247, 118, 142);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(65, 72, 104);
        palette[PaletteColor::HighlightText] = Color::Rgb(36, 40, 59);
        palette[PaletteStyle::Background] = Style::from(Black.dark());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Blue.light());
        palette[PaletteStyle::TitlePrimary] = Style::from(Magenta.dark()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Magenta.light());
        palette[PaletteStyle::Highlight] = Style::from(Red.light()).combine(Effect::Bold);
        palette[PaletteStyle::HighlightInactive] = Style::from(Blue.dark());
        palette[PaletteStyle::Shadow] = Style::from(Black.dark());
    })
}

/// Returns a "Tokyo Night Light" theme for the application.
///
/// This theme provides a lighter variation of the Tokyo Night color scheme.
pub fn tokyo_night_light_theme() -> Theme {
    build_theme(false, BorderStyle::Outset, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(213, 214, 219);
        palette[PaletteColor::Shadow] = Color::Rgb(201, 201, 209);
        palette[PaletteColor::View] = Color::Rgb(213, 214, 219);
        palette[PaletteColor::Primary] = Color::Rgb(55, 96, 191);
        palette[PaletteColor::Secondary] = Color::Rgb(126, 156, 216);
        palette[PaletteColor::Tertiary] = Color::Rgb(161, 161, 170);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(152, 84, 241);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(192, 169, 247);
        palette[PaletteColor::Highlight] = Color::Rgb(247, 118, 142);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(188, 176, 194);
        palette[PaletteColor::HighlightText] = Color::Rgb(213, 214, 219);
        palette[PaletteStyle::Background] = Style::from(White.dark());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(Black.dark());
        palette[PaletteStyle::Secondary] = Style::from(Blue.light());
        palette[PaletteStyle::TitlePrimary] = Style::from(Magenta.light()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Magenta.dark());
        palette[PaletteStyle::Highlight] = Style::from(Red.dark()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(White.dark());
        palette[PaletteStyle::Shadow] = Style::none();
    })
}

/// Returns a "Material Palenight" theme for the application.
///
/// This theme is inspired by Material Palenight, featuring soft pastel accents on a dark background.
pub fn material_palenight_theme() -> Theme {
    build_theme(true, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(41, 45, 62);
        palette[PaletteColor::Shadow] = Color::Rgb(28, 31, 43);
        palette[PaletteColor::View] = Color::Rgb(41, 45, 62);
        palette[PaletteColor::Primary] = Color::Rgb(166, 172, 205);
        palette[PaletteColor::Secondary] = Color::Rgb(195, 232, 141);
        palette[PaletteColor::Tertiary] = Color::Rgb(247, 140, 108);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(130, 170, 255);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(199, 146, 234);
        palette[PaletteColor::Highlight] = Color::Rgb(137, 221, 255);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(103, 110, 149);
        palette[PaletteColor::HighlightText] = Color::Rgb(41, 45, 62);
        palette[PaletteStyle::Background] = Style::from(Black.dark());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Green.light());
        palette[PaletteStyle::TitlePrimary] = Style::from(Blue.light()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Magenta.light());
        palette[PaletteStyle::Highlight] = Style::from(Cyan.light()).combine(Effect::Bold);
        palette[PaletteStyle::HighlightInactive] = Style::from(Blue.dark());
        palette[PaletteStyle::Shadow] = Style::from(Black.dark());
    })
}

/// Returns a "Zenburn" theme for the application.
///
/// This theme applies the Zenburn color palette, known for its low-contrast, warm, and muted colors.
pub fn zenburn_theme() -> Theme {
    build_theme(true, BorderStyle::None, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(63, 63, 63);
        palette[PaletteColor::Shadow] = Color::Rgb(41, 41, 41);
        palette[PaletteColor::View] = Color::Rgb(63, 63, 63);
        palette[PaletteColor::Primary] = Color::Rgb(220, 220, 204);
        palette[PaletteColor::Secondary] = Color::Rgb(111, 111, 111);
        palette[PaletteColor::Tertiary] = Color::Rgb(112, 144, 128);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(223, 175, 143);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(240, 223, 175);
        palette[PaletteColor::Highlight] = Color::Rgb(140, 208, 211);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(95, 127, 95);
        palette[PaletteColor::HighlightText] = Color::Rgb(63, 63, 63);
        palette[PaletteStyle::Background] = Style::from(Black.dark());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Color::Rgb(127, 127, 127));
        palette[PaletteStyle::TitlePrimary] = Style::from(Yellow.dark()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Yellow.light());
        palette[PaletteStyle::Highlight] = Style::from(Cyan.light()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Green.dark());
        palette[PaletteStyle::Shadow] = Style::from(Black.dark());
    })
}

/// Returns an "Arc Dark" theme for the application.
///
/// This theme is influenced by the Arc Dark color palette, featuring cool grays with bright accent colors.
pub fn arc_dark_theme() -> Theme {
    build_theme(true, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(47, 52, 63);
        palette[PaletteColor::Shadow] = Color::Rgb(35, 37, 43);
        palette[PaletteColor::View] = Color::Rgb(47, 52, 63);
        palette[PaletteColor::Primary] = Color::Rgb(211, 218, 227);
        palette[PaletteColor::Secondary] = Color::Rgb(156, 168, 179);
        palette[PaletteColor::Tertiary] = Color::Rgb(124, 138, 150);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(82, 148, 226);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(170, 209, 247);
        palette[PaletteColor::Highlight] = Color::Rgb(91, 144, 191);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(79, 91, 102);
        palette[PaletteColor::HighlightText] = Color::Rgb(47, 52, 63);
        palette[PaletteStyle::Background] = Style::from(Black.dark());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Color::Rgb(127, 127, 127));
        palette[PaletteStyle::TitlePrimary] = Style::from(Blue.light()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Blue.dark()).combine(Effect::Italic);
        palette[PaletteStyle::Highlight] = Style::from(Blue.light()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Black.light());
        palette[PaletteStyle::Shadow] = Style::from(Black.dark());
    })
}

/// Returns a "Pokémon" theme for the application.
///
/// This theme uses a vibrant, colorful palette inspired by the Pokémon franchise.
pub fn pokemon_theme() -> Theme {
    build_theme(false, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(255, 255, 255);
        palette[PaletteColor::Shadow] = Color::Rgb(220, 220, 220);
        palette[PaletteColor::View] = Color::Rgb(255, 255, 255);
        palette[PaletteColor::Primary] = Color::Rgb(45, 45, 45);
        palette[PaletteColor::Secondary] = Color::Rgb(240, 0, 0);
        palette[PaletteColor::Tertiary] = Color::Rgb(248, 208, 48);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(59, 76, 202);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(255, 203, 5);
        palette[PaletteColor::Highlight] = Color::Rgb(240, 0, 0);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(200, 50, 50);
        palette[PaletteColor::HighlightText] = Color::Rgb(255, 255, 255);
        palette[PaletteStyle::Background] = Style::none();
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(Black.dark());
        palette[PaletteStyle::Secondary] = Style::from(Red.light());
        palette[PaletteStyle::TitlePrimary] = Style::from(Blue.light()).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Yellow.dark()).combine(Effect::Italic);
        palette[PaletteStyle::Highlight] = Style::from(Red.dark()).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Red.light());
        palette[PaletteStyle::Shadow] = Style::from(Color::Rgb(127, 127, 127));
    })
}

/// Returns a "Pokémon Dark" theme for the application.
///
/// This theme features a dark background with accent colors inspired by Pokémon.
pub fn pokemon_dark_theme() -> Theme {
    build_theme(true, BorderStyle::Simple, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(0, 0, 0);
        palette[PaletteColor::Shadow] = Color::Rgb(14, 52, 112);
        palette[PaletteColor::View] = Color::Rgb(0, 0, 0);
        palette[PaletteColor::Primary] = Color::Rgb(255, 255, 255);
        palette[PaletteColor::Secondary] = Color::Rgb(52, 96, 174);
        palette[PaletteColor::Tertiary] = Color::Rgb(237, 21, 30);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(255, 204, 1);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(200, 160, 1);
        palette[PaletteColor::Highlight] = Color::Rgb(237, 21, 30);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(168, 5, 10);
        palette[PaletteColor::HighlightText] = Color::Rgb(255, 255, 255);
        palette[PaletteStyle::Background] = Style::from(Black.dark());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(White.dark());
        palette[PaletteStyle::Secondary] = Style::from(Color::Rgb(52, 96, 174));
        palette[PaletteStyle::TitlePrimary] =
            Style::from(Color::Rgb(255, 204, 1)).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Color::Rgb(200, 160, 1));
        palette[PaletteStyle::Highlight] =
            Style::from(Color::Rgb(237, 21, 30)).combine(Effect::Bold);
        palette[PaletteStyle::HighlightInactive] = Style::from(Color::Rgb(168, 5, 10));
        palette[PaletteStyle::Shadow] = Style::from(Color::Rgb(14, 52, 112));
    })
}

/// Returns a "Pokémon Light" theme for the application.
///
/// This theme provides a bright variation with colors reminiscent of the Pokémon franchise.
pub fn pokemon_light_theme() -> Theme {
    build_theme(false, BorderStyle::Outset, |palette| {
        palette[PaletteColor::Background] = Color::Rgb(255, 255, 255);
        palette[PaletteColor::Shadow] = Color::Rgb(200, 160, 1);
        palette[PaletteColor::View] = Color::Rgb(255, 255, 255);
        palette[PaletteColor::Primary] = Color::Rgb(0, 0, 0);
        palette[PaletteColor::Secondary] = Color::Rgb(52, 96, 174);
        palette[PaletteColor::Tertiary] = Color::Rgb(237, 21, 30);
        palette[PaletteColor::TitlePrimary] = Color::Rgb(255, 204, 1);
        palette[PaletteColor::TitleSecondary] = Color::Rgb(200, 160, 1);
        palette[PaletteColor::Highlight] = Color::Rgb(237, 21, 30);
        palette[PaletteColor::HighlightInactive] = Color::Rgb(168, 5, 10);
        palette[PaletteColor::HighlightText] = Color::Rgb(255, 255, 255);
        palette[PaletteStyle::Background] = Style::from(White.light());
        palette[PaletteStyle::View] = Style::none();
        palette[PaletteStyle::Primary] = Style::from(Black.dark());
        palette[PaletteStyle::Secondary] = Style::from(Color::Rgb(52, 96, 174));
        palette[PaletteStyle::TitlePrimary] =
            Style::from(Color::Rgb(255, 204, 1)).combine(Effect::Bold);
        palette[PaletteStyle::TitleSecondary] = Style::from(Color::Rgb(200, 160, 1));
        palette[PaletteStyle::Highlight] =
            Style::from(Color::Rgb(237, 21, 30)).combine(Effect::Reverse);
        palette[PaletteStyle::HighlightInactive] = Style::from(Color::Rgb(168, 5, 10));
        palette[PaletteStyle::Shadow] = Style::from(Color::Rgb(200, 160, 1));
    })
}

/// Returns a vector of all available theme-generating functions.
///
/// The vector includes functions for both dark and light themes.
pub fn themes() -> Vec<fn() -> Theme> {
    vec![
        pokemon_dark_theme,
        pokemon_light_theme,
        pokemon_theme,
        arc_dark_theme,
        dark_forest_theme,
        dracula_theme,
        gruvbox_dark_theme,
        material_palenight_theme,
        midnight_theme,
        molokai_dark_theme,
        monokai_theme,
        nord_theme,
        oceanic_dark_theme,
        one_dark_theme,
        solarized_dark_theme,
        tokyo_night_storm_theme,
        zenburn_theme,
        github_light_theme,
        gruvbox_light_theme,
        nord_light_theme,
        papercolor_light_theme,
        solarized_light_theme,
        tokyo_night_light_theme,
    ]
}
