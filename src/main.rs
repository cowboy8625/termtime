use humantime::format_duration;

use clap::{crate_authors, crate_description, crate_name, crate_version, Arg, Command};
use crossterm::{
    cursor, event, execute, queue, style,
    style::Stylize,
    terminal,
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};
use rand::{thread_rng, Rng};
use std::io::{Stdout, Write};
use std::time::{Duration, Instant};

const FONTS: [&str; 39] = [
    "ascii9",
    "ascii12",
    "banner",
    "big",
    "bigascii9",
    "bigascii12",
    "bigmono9",
    "bigmono12",
    "block",
    "bubble",
    "circle",
    "digital",
    "emboss",
    "emboss2",
    "future",
    "ivrit",
    "lean",
    "letter",
    "mini",
    "mnemonic",
    "mono9",
    "mono12",
    "pagga",
    "script",
    "shadow",
    "slant",
    "small",
    "smascii9",
    "smascii12",
    "smblock",
    "smbraille",
    "smmono9",
    "smmono12",
    "smscript",
    "smshadow",
    "smslant",
    "standard",
    "term",
    "wideterm",
];

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
    font: usize,
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
}

impl Flags {
    fn fg(&self) -> style::Color {
        self.fg.into()
    }

    fn bg(&self) -> style::Color {
        self.bg.into()
    }
}

fn args() -> Flags {
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
                .help(&*format!("Set font name {:#?}", FONTS))
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
        .get_matches();
    let font = matches.value_of("font").unwrap_or("").to_string();

    let random = matches.is_present("random");
    let font = if random {
        thread_rng().gen_range(0..FONTS.len())
    } else {
        FONTS
            .iter()
            .enumerate()
            .find(|(_, i)| i == &&font)
            .map(|(i, _)| i)
            .unwrap_or(0)
    };
    Flags {
        message: matches.value_of("message").unwrap_or("").to_string(),

        font,
        fg: matches
            .value_of("foreground")
            .unwrap_or("0,255,0")
            .into_tuple(),
        bg: matches
            .value_of("background")
            .unwrap_or("40,40,40")
            .into_tuple(),
    }
}

fn figet_message(msg: &str, font: usize) -> String {
    String::from_utf8(
        std::process::Command::new("figlet")
            .args(&["-f", FONTS[font]])
            // .args(&["-w", "100"])
            // .arg("-t")
            .arg("-c")
            .arg("-n")
            .arg(msg)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
}

#[derive(Debug, Clone, Copy)]
enum Alignment {
    Top = 3,
    Center = 2,
    Bottom = 1,
}

fn printer(
    stdout: &mut Stdout,
    width: u16,
    height: u16,
    message: &str,
    flags: &Flags,
    align: Alignment,
) -> Result<()> {
    let message = message
        .lines()
        .filter(|l| !l.chars().all(|c| c == ' '))
        .map(|s| format!("{}\n", s))
        .collect::<String>();
    let text_width = message.lines().map(|x| x.len()).max().unwrap_or(0) as u16;
    let text_height = message.lines().count() as u16;
    for (i, line) in message.lines().enumerate() {
        queue!(
            stdout,
            cursor::MoveTo(
                (width / 2) - (text_width / 2),
                (height / align as u16) - (text_height / 2) + i as u16
            ),
            style::Print(&format!("{:^1}", line.with(flags.fg()).on(flags.bg())))
        )?;
    }
    Ok(())
}
fn clear(stdout: &mut Stdout) -> Result<()> {
    queue!(stdout, terminal::Clear(terminal::ClearType::All),)
}

fn display(
    stdout: &mut Stdout,
    width: u16,
    height: u16,
    start: Instant,
    flags: &Flags,
) -> Result<()> {
    clear(stdout)?;
    let message = figet_message(&flags.message, flags.font);
    printer(stdout, width, height, &message, &flags, Alignment::Top)?;
    // queue!(
    //
    //     stdout,
    //     cursor::MoveTo((width / 2) - (message_len / 2), height / 3),
    //     style::Print(&flags.message)
    // )?;
    let time = format_duration(Duration::from_secs(start.elapsed().as_secs())).to_string();
    let fig_string = figet_message(&time, flags.font);
    printer(
        stdout,
        width,
        height,
        &fig_string,
        &flags,
        Alignment::Center,
    )?;
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let flags = args();
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    let (width, height) = terminal::size()?;
    execute!(&mut stdout, cursor::Hide, terminal::EnterAlternateScreen)?;
    let start = Instant::now();
    display(&mut stdout, width, height, start, &flags)?;
    loop {
        if event::poll(Duration::from_secs(1))? {
            let events = event::read()?;
            if matches!(
                events,
                event::Event::Key(event::KeyEvent {
                    code: event::KeyCode::Esc,
                    modifiers: event::KeyModifiers::NONE
                })
            ) {
                break;
            }
        }
        display(&mut stdout, width, height, start, &flags)?;
    }
    execute!(&mut stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
