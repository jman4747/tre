use super::diagram_formatting::FormattedEntry;
use lscolors::{self, LsColors, Style};
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

fn color_print<T>(text: T, color: Option<&ColorSpec>) -> bool
where
    T: Display,
{
    if let Some(color_spec) = color {
        let stdout = BufferWriter::stdout(ColorChoice::Auto);
        let mut buffer = stdout.buffer();
        buffer
            .set_color(color_spec)
            .and_then(|_| write!(&mut buffer, "{}", text))
            .and_then(|_| buffer.reset())
            .and_then(|_| stdout.print(&buffer))
            .is_ok()
    } else {
        print!("{}", text);
        true
    }
}

pub fn print_entries(entries: &[FormattedEntry], create_alias: bool, lscolors: Option<&LsColors>) {
    let number_color = lscolors.map(|_| ColorSpec::new().set_fg(Some(Color::Red)).to_owned());
    for (index, entry) in entries.iter().enumerate() {
        if create_alias {
            print!("{}[", entry.prefix);

            color_print(index, number_color.as_ref());
            print!("] ");
        } else {
            print!("{}", entry.prefix);
        }

        let spec = lscolors.map(|c| {
            c.style_for_path(&entry.path)
                .map(convert_to_color_spec)
                .unwrap_or_default()
        });
        color_print(&entry.name, spec.as_ref());
        println!()
    }
}

fn convert_color(color: &lscolors::Color) -> Color {
    match color {
        lscolors::Color::RGB(r, g, b) => Color::Rgb(*r, *g, *b),
        lscolors::Color::Fixed(n) => Color::Ansi256(*n),
        lscolors::Color::Black => Color::Black,
        lscolors::Color::Red => Color::Red,
        lscolors::Color::Green => Color::Green,
        lscolors::Color::Yellow => Color::Yellow,
        lscolors::Color::Blue => Color::Blue,
        lscolors::Color::Magenta => Color::Magenta,
        lscolors::Color::Cyan => Color::Cyan,
        lscolors::Color::White => Color::White,

        // bright colors don't have a direct alternative in termcolor::color
        // translate them to "ansi 256" colors using the same value as used
        // in to_ansi_term_color
        lscolors::Color::BrightBlack => Color::Ansi256(8),
        lscolors::Color::BrightRed => Color::Ansi256(9),
        lscolors::Color::BrightGreen => Color::Ansi256(10),
        lscolors::Color::BrightYellow => Color::Ansi256(11),
        lscolors::Color::BrightBlue => Color::Ansi256(12),
        lscolors::Color::BrightMagenta => Color::Ansi256(13),
        lscolors::Color::BrightCyan => Color::Ansi256(14),
        lscolors::Color::BrightWhite => Color::Ansi256(15),
    }
}

fn convert_to_color_spec(style: &Style) -> ColorSpec {
    let mut spec = ColorSpec::new();

    if let Some(color) = &style.foreground {
        spec.set_fg(Some(convert_color(color)));
    }

    if let Some(color) = &style.background {
        spec.set_bg(Some(convert_color(color)));
    }

    spec.set_bold(style.font_style.bold);
    spec.set_italic(style.font_style.italic);
    spec.set_underline(style.font_style.underline);

    spec
}

#[cfg(target_os = "windows")]
fn open_alias_file_with_suffix(suffix: &str) -> io::Result<File> {
    let file_name = format!(
        "tre_aliases_{}.{}",
        env::var("USERNAME").unwrap_or_else(|_| "".to_string()),
        suffix
    );
    let home = env::var("HOME").unwrap_or_else(|_| r".".to_string());
    let tmp = env::var("TEMP").unwrap_or(home);
    let path: PathBuf = [tmp, file_name].iter().collect();
    let file = File::create(&path);
    if file.is_err() {
        eprintln!("[tre] failed to open {:?}", path);
    }

    file
}

#[cfg(target_os = "windows")]
pub fn create_edit_aliases(editor: &str, entries: &[FormattedEntry]) {
    let powershell_alias = open_alias_file_with_suffix("psm1");
    if let Ok(mut alias_file) = powershell_alias {
        for (index, entry) in entries.iter().enumerate() {
            let result = if editor.is_empty() {
                writeln!(
                    &mut alias_file,
                    "Function e{} {{ Start-Process \"{}\"}}",
                    index, entry.path
                )
            } else {
                writeln!(
                    &mut alias_file,
                    "Function e{} {{ {} $args \"{}\"}}",
                    index, editor, entry.path
                )
            };
            if let Err(e) = result {
                eprintln!("[tre] failed to write to PowerShell alias file due to:");
                eprintln!("{e}");
                return
            }
        }
        let result = writeln!(
            &mut alias_file,
            "Export-ModuleMember -Function *"
        );
        if let Err(e) = result {
            eprintln!("[tre] failed to write to PowerShell alias file due to:");
            eprintln!("{e}");
            return
        }
    }

    let cmd_alias = open_alias_file_with_suffix("bat");
    if let Ok(mut alias_file) = cmd_alias {
        for (index, entry) in entries.iter().enumerate() {
            let editor = if editor.is_empty() { "START" } else { editor };
            let result = writeln!(
                &mut alias_file,
                "doskey /exename=cmd.exe e{}={} {}",
                index, editor, entry.path,
            );

            if let Err(e) = result {
                eprintln!("[tre] failed to write to CMD alias file due to:");
                eprintln!("{e}");
                return
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn open_alias_file() -> io::Result<File> {
    let user = env::var("USER").unwrap_or_else(|_| "".to_string());
    let alias_file = format!("/tmp/tre_aliases_{}", &user);
    let path: PathBuf = [alias_file].iter().collect();
    let file = File::create(&path);
    if file.is_err() {
        eprintln!("[tre] failed to open {:?}", path);
    }

    file
}

#[cfg(not(target_os = "windows"))]
pub fn create_edit_aliases(editor: &str, entries: &[FormattedEntry]) {
    let alias = open_alias_file();
    if let Ok(mut alias_file) = alias {
        for (index, entry) in entries.iter().enumerate() {
            let result = writeln!(
                &mut alias_file,
                "alias e{}=\"eval '{} \\\"{}\\\"'\"",
                index,
                editor,
                entry.path.replace('\'', "\\'")
            );

            if result.is_err() {
                eprintln!("[tre] failed to write to alias file.");
            }
        }
    }
}
