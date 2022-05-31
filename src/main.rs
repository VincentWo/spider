use clap::Parser;
use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nix::{sys::{ptrace, wait::waitpid}, unistd::Pid};
use render::{RenderUpdate::{CommandWindow, self}, CommandUpdate};
use tui::{
    backend::CrosstermBackend, Terminal,
};
use tokio::{fs::File as TokioFile, io::AsyncReadExt};


use std::{io::{self, Read}, process, os::unix::prelude::{CommandExt, IntoRawFd, FromRawFd}, fs::File};

mod render;
mod state_manager;

#[derive(Parser, Debug)]
struct Args {
    executable: String,
    args: Vec<String>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut command = process::Command::new(args.executable);
    command.args(args.args);
    command.stdout(process::Stdio::piped()).stdin(process::Stdio::piped());
    unsafe {
        // SAFETY
        // This is only unsafe cause it allows breaking certain preconditions
        // But as long as we aren't passing anything complicated or with custom Drop
        // we are fine
        // command.pre_exec(|| Ok(ptrace::traceme()?));
    }
    let mut child = command.spawn().unwrap();
    let mut child_stdout = unsafe {
        TokioFile::from_raw_fd(child.stdout.take().unwrap().into_raw_fd())
    };

    tokio::spawn(async move {
        let mut buffer = [0u8; 256];
        
        loop {
            let count = child_stdout.read(&mut buffer).await.unwrap();
            let data = &buffer[..count];

            let s = std::str::from_utf8(data).unwrap();

            println!("{}", s);
        }
    }).await.unwrap();

    let pid = Pid::from_raw(child.id() as i32);

    let stdin_send = send.clone();
    std::thread::spawn(move || {
        let send = stdin_send;
        let mut buffer = [0u8; 1024];

        loop {
            match child_stdout.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => break,
                Err(_) => todo!(),
            }
        }
    });

    waitpid(Some(pid), None).unwrap();

    ptrace::cont(pid, None)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    std::thread::spawn(move || {
        render::start_render_loop(terminal, recv).await.unwrap();
        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).unwrap();
    });

    loop {
        match read()? {
            crossterm::event::Event::Key(key) => {
                if key.modifiers == KeyModifiers::CONTROL {
                    match key.code {
                        crossterm::event::KeyCode::Char('c') => break,
                        _ => continue,
                    }
                }
                let result = match key.code {
                    crossterm::event::KeyCode::Backspace => send.send(RenderUpdate::CommandWindow(CommandUpdate::Backspace)),
                    crossterm::event::KeyCode::Enter => send.send(RenderUpdate::CommandWindow(CommandUpdate::Newline)),
                    crossterm::event::KeyCode::Left => todo!(),
                    crossterm::event::KeyCode::Right => todo!(),
                    crossterm::event::KeyCode::Up => todo!(),
                    crossterm::event::KeyCode::Down => todo!(),
                    crossterm::event::KeyCode::Home => todo!(),
                    crossterm::event::KeyCode::End => todo!(),
                    crossterm::event::KeyCode::PageUp => todo!(),
                    crossterm::event::KeyCode::PageDown => todo!(),
                    crossterm::event::KeyCode::Tab => todo!(),
                    crossterm::event::KeyCode::BackTab => todo!(),
                    crossterm::event::KeyCode::Delete => todo!(),
                    crossterm::event::KeyCode::Insert => todo!(),
                    crossterm::event::KeyCode::F(_) => todo!(),
                    crossterm::event::KeyCode::Char(c) => send.send(RenderUpdate::CommandWindow(CommandUpdate::NewChar(c))),
                    crossterm::event::KeyCode::Null => todo!(),
                    crossterm::event::KeyCode::Esc => todo!(),
                };
                if let Err(err) = result {
                    println!("{err:#?}");
                    break;
                }
            }
            crossterm::event::Event::Mouse(_) => {}
            crossterm::event::Event::Resize(_, _) => {}
        };
    }
    send.send(RenderUpdate::Shutdown).unwrap();
    Ok(())
}
