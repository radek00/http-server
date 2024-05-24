use std::{
    error::Error,
    io::{self, Write},
};

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

pub struct Logger {
    out_writer: BufferWriter,
}

impl Logger {
    pub fn new() -> Logger {
        Logger {
            out_writer: BufferWriter::stdout(ColorChoice::Auto),
        }
    }

    pub fn print(
        &self,
        fmtstr: &str,
        args: Vec<(String, Option<Color>)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = self.out_writer.buffer();
        let mut arg_iter = args.iter();
        let mut char_iter = fmtstr.chars();
        let mut current = char_iter.next();
        let mut count = 0;
        let mut color_spec = ColorSpec::new();
        while let Some(c) = current {
            match c {
                '{' => {
                    let c = char_iter.next();
                    match c {
                        Some('}') => {
                            if let Some((s, color)) = arg_iter.next() {
                                if !s.is_empty() {
                                    color_spec.set_fg(*color);
                                    buffer.set_color(&color_spec)?;
                                    buffer.write_all(s.as_bytes())?;
                                    if color.is_some() {
                                        buffer.reset()?;
                                    }
                                }
                                count += 1;
                            } else {
                                return Err(Box::from(format!(
                                    "Not enough arguments (need more than {})",
                                    count
                                )));
                            }
                        }
                        Some('{') => {
                            buffer.write_all(b"{")?;
                        }
                        _ => {
                            return Err(Box::from("{{ not closed"));
                        }
                    }
                }
                '}' => {
                    let c = char_iter.next();
                    match c {
                        Some('}') => {
                            buffer.write_all(b"}")?;
                        }
                        _ => {
                            return Err(Box::from("}} not closed"));
                        }
                    }
                }
                c => {
                    let mut buf = [0; 4];
                    buffer.write_all(c.encode_utf8(&mut buf).as_bytes())?;
                }
            }
            current = char_iter.next();
        }
        buffer.write_all(b"\n")?;
        self.out_writer.print(&buffer)?;
        Ok(())
    }
}
