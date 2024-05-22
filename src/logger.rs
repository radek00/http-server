use std::io::{self, Write};

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

fn get_status_code_color(status_code: u16) -> Color {
    match status_code {
        100..=199 => Color::Cyan,
        200..=299 => Color::Green,
        300..=399 => Color::Yellow,
        400..=499 => Color::Red,
        _ => Color::Magenta,
    }
}

pub struct Logger {
    out_writer: BufferWriter,
}

impl Logger {
    pub fn new() -> Logger {
        Logger {
            out_writer: BufferWriter::stdout(ColorChoice::Auto),
        }
    }

    pub fn log(&self, status_code: u16, path: &str, method: &str) -> io::Result<()> {
        let mut buffer = self.out_writer.buffer();

        buffer.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
        let time = chrono::offset::Local::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        write!(&mut buffer, "{}", format!("{} - ", time))?;

        buffer.set_color(ColorSpec::new().set_fg(Some(get_status_code_color(status_code))))?;
        write!(&mut buffer, "{}", status_code)?;

        buffer.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
        writeln!(&mut buffer, " - {} {}", method, path)?;

        self.out_writer.print(&buffer)
    }
}
