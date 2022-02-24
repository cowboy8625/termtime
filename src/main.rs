use humantime::format_duration;

use clap::{crate_authors, crate_description, crate_name, crate_version, Arg, Command};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue, style,
    style::Stylize,
    terminal,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::{thread_rng, Rng};
use std::io::{Stdout, Write};
use std::time::{Duration, Instant};

impl StrTuple<(u64, u64)> for String {
    fn into_tuple(self) -> (u64, u64) {
        let mut nums = Vec::new();
        for num in self.split(',') {
            nums.push(
                num.parse::<u64>()
                    .expect("This is not the correct format, expecting 0,0,0 or name like white"),
            );
        }
        let a = nums[0];
        let b = nums[1];
        (a, b)
    }
}

impl StrTuple<(u8, u8, u8)> for String {
    fn into_tuple(self) -> (u8, u8, u8) {
        let mut nums = Vec::new();
        for num in self.split(',') {
            nums.push(
                num.parse::<u8>()
                    .expect("This is not the correct format, expecting 0,0,0 or name like white"),
            );
        }
        let a = nums[0];
        let b = nums[1];
        let c = nums[2];
        (a, b, c)
    }
}

impl StrTuple<(u8, u8, u8)> for &str {
    fn into_tuple(self) -> (u8, u8, u8) {
        let mut nums = Vec::new();
        for num in self.split(',') {
            nums.push(
                num.parse::<u8>()
                    .expect("This is not the correct format, expecting 0,0,0 or name like white"),
            );
        }
        let a = nums[0];
        let b = nums[1];
        let c = nums[2];
        (a, b, c)
    }
}

trait StrTuple<T> {
    fn into_tuple(self) -> T;
}

struct Flags {
    message: String,
    algin_message: Alignment,
    font: String,
    fg: Option<(u8, u8, u8)>,
    bg: Option<(u8, u8, u8)>,
}

impl Flags {
    fn fg(&self) -> Option<style::Color> {
        self.fg.map(Into::into)
    }

    fn bg(&self) -> Option<style::Color> {
        self.bg.map(Into::into)
    }

    fn algin_clock(&self) -> Alignment {
        match self.algin_message {
            Alignment::Top => Alignment::Center,
            Alignment::Bottom => Alignment::Center,
            Alignment::Center => Alignment::Top,
        }
    }
}

fn args() -> Result<Flags, Box<dyn std::error::Error>> {
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::new("message")
                .short('m')
                .long("msg")
                .help("Message to display")
                .takes_value(true),
        )
        .arg(
            Arg::new("font")
                .short('f')
                .long("font")
                .help(&*format!("Set font name font name."))
                .takes_value(true),
        )
        .arg(
            Arg::new("foreground")
                .short('F')
                .long("fg")
                .help("Set font foreground color")
                .takes_value(true),
        )
        .arg(
            Arg::new("background")
                .short('B')
                .long("bg")
                .help("Set font background color")
                .takes_value(true),
        )
        .arg(
            Arg::new("random")
                .short('r')
                .long("rand")
                .help(
                    "Sets Font to be random
WARNING over rides -f flag",
                )
                .takes_value(false),
        )
        .arg(
            Arg::new("algin_message")
                .short('a')
                .long("algin")
                .help(
                    "Sets the Message
    Option's
        top,
        center,
        bottom",
                )
                .takes_value(true),
        )
        .get_matches();

    let random = matches.is_present("random");
    let font = if random {
        let fonts = font_list()?;
        let idx = thread_rng().gen_range(0..fonts.len());
        fonts[idx].to_string()
    } else {
        matches.value_of("font").unwrap_or("slant").to_owned()
    };

    Ok(Flags {
        message: matches.value_of("message").unwrap_or("").to_string(),
        algin_message: match matches.value_of("algin_message").unwrap_or("") {
            "center" => Alignment::Center,
            "bottom" => Alignment::Bottom,
            _ => Alignment::Top,
        },
        font,
        fg: matches.value_of("foreground").map(StrTuple::into_tuple),
        bg: matches.value_of("background").map(StrTuple::into_tuple),
    })
}

fn figet_message(
    msg: &str,
    flags: &Flags,
    width: u16,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(String::from_utf8(
        std::process::Command::new("figlet")
            .args(&["-f", flags.font.as_str()])
            .args(&["-w", width.to_string().as_str()])
            // .arg("-t")
            .arg("-c")
            // .arg("-n")
            .arg(msg)
            .output()?
            .stdout,
    )?)
}

#[derive(Debug, Clone, Copy)]
enum Alignment {
    Top = 3,
    Center = 2,
    Bottom = 1,
}

fn rpad(msg: &str) -> String {
    msg.lines()
        .map(|line| format!("{}          \n", line))
        .collect()
}

// fn remove_empty_lines(msg: &str) -> String {
//     msg.lines()
//         .filter(|l| !l.chars().all(|c| c == ' '))
//         .map(|s| format!("{}\n", s))
//         .collect::<String>()
// }

fn printer(
    stdout: &mut Stdout,
    height: u16,
    message: &str,
    flags: &Flags,
    align: Alignment,
) -> Result<(), Box<dyn std::error::Error>> {
    let message = rpad(message);
    let text_height = message.lines().count() as u16;
    let y = match align {
        Alignment::Top => 0,
        Alignment::Center => height / 2 - text_height / 2,
        Alignment::Bottom => height - text_height,
    };

    for (i, line) in message.lines().enumerate() {
        let line = match (flags.fg(), flags.bg()) {
            (Some(fg), Some(bg)) => line.with(fg).on(bg),
            (Some(fg), None) => line.with(fg),
            (None, Some(bg)) => line.on(bg),
            _ => crossterm::style::StyledContent::new(crossterm::style::ContentStyle::new(), line),
        };
        queue!(stdout, cursor::MoveTo(0, y + i as u16), style::Print(&line))?;
    }
    Ok(())
}
fn clear(stdout: &mut Stdout) -> Result<(), Box<dyn std::error::Error>> {
    Ok(queue!(stdout, terminal::Clear(terminal::ClearType::All),)?)
}

fn display(
    stdout: &mut Stdout,
    width: u16,
    height: u16,
    start: Instant,
    flags: &Flags,
) -> Result<(), Box<dyn std::error::Error>> {
    // clear(stdout)?;
    let message = figet_message(&flags.message, &flags, width)?;
    printer(stdout, height, &message, &flags, flags.algin_message)?;
    let time = format_duration(Duration::from_secs(start.elapsed().as_secs())).to_string();
    let fig_string = figet_message(&time, &flags, width)?;
    printer(stdout, height, &fig_string, &flags, flags.algin_clock())?;
    stdout.flush()?;
    Ok(())
}

fn events_system(
    stdout: &mut Stdout,
    width: &mut u16,
    height: &mut u16,
    running: &mut bool,
    //flags: &mut Flags,
) -> Result<(), Box<dyn std::error::Error>> {
    if event::poll(Duration::from_secs(1))? {
        let events = event::read()?;
        match events {
            Event::Key(key_event) => match key_event {
                KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                } => {
                    *running = false;
                }
                // KeyEvent {
                //     code: KeyCode::Up,
                //     modifiers: KeyModifiers::NONE,
                // } => {
                //     flags.font = (flags.font + 1) % font_len;
                //     clear(stdout)?;
                // }
                // KeyEvent {
                //     code: KeyCode::Down,
                //     modifiers: KeyModifiers::NONE,
                // } => {
                //     flags.font = (flags.font - 1) % fonts_len;
                //     clear(stdout)?;
                // }
                _ => {}
            },
            Event::Resize(w, h) => {
                *width = w;
                *height = h;
                clear(stdout)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn font_list() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use std::ffi::OsStr;
    let paths = std::fs::read_dir("/usr/share/figlet/")?;

    let mut font_names = Vec::new();
    for dir_entry in paths {
        let path = dir_entry?.path();
        if path.extension() == Some(&OsStr::new("tlf")) {
            match path.file_stem() {
                Some(name) => font_names.push(name.to_string_lossy().to_string()),
                None => {}
            }
        }
    }
    Ok(font_names)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let font_len = font_list()?.len();
    let flags = args()?;
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    let (mut width, mut height) = terminal::size()?;
    execute!(&mut stdout, cursor::Hide, terminal::EnterAlternateScreen)?;
    let start = Instant::now();
    display(&mut stdout, width, height, start, &flags)?;
    let mut running = true;
    while running {
        events_system(
            &mut stdout,
            &mut width,
            &mut height,
            &mut running,
            //    &mut flags,
        )?;
        display(&mut stdout, width, height, start, &flags)?;
    }
    execute!(&mut stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
