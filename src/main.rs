use humantime::format_duration;

use crossterm::{
    cursor, event, execute, queue, style, terminal,
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

fn display(
    stdout: &mut Stdout,
    width: u16,
    height: u16,
    start: Instant,
    font: usize,
) -> Result<()> {
    let time = format_duration(Duration::from_secs(start.elapsed().as_secs())).to_string();
    let output = std::process::Command::new("figlet")
        .args(&["-f", FONTS[font]])
        .arg("-n")
        .arg(time)
        .output()
        .unwrap();
    let fig_string = String::from_utf8(output.stdout).unwrap();
    let text_width = fig_string.lines().map(|x| x.len()).max().unwrap_or(0) as u16;
    let text_height = fig_string.lines().count() as u16;
    queue!(stdout, terminal::Clear(terminal::ClearType::All),)?;
    for (i, line) in fig_string.lines().enumerate() {
        queue!(
            stdout,
            cursor::MoveTo(
                (width / 2) - (text_width / 2),
                (height / 2) - (text_height / 2) + i as u16
            ),
            style::Print(&format!("{:^1}", line))
        )?;
    }
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let font = thread_rng().gen_range(0..FONTS.len());
    let mut stdout = std::io::stdout();
    let (width, height) = terminal::size()?;
    execute!(&mut stdout, cursor::Hide, terminal::EnterAlternateScreen)?;
    let start = Instant::now();
    display(&mut stdout, width, height, start, font)?;
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
        display(&mut stdout, width, height, start, font)?;
    }
    execute!(&mut stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
