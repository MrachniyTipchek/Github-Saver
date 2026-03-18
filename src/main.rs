use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{execute, queue};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Deserialize, Clone)]
struct Repo {
    name: String,
    full_name: String,
    clone_url: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
    }
}

fn run() -> anyhow::Result<()> {
    loop {
        let choice = main_menu_tui()?;
        match choice {
            MenuChoice::RunBackup => {
                start_backup_flow()?;
            }
            MenuChoice::Exit => {
                break;
            }
        }
    }
    Ok(())
}

enum MenuChoice {
    RunBackup,
    Exit,
}

fn main_menu_tui() -> anyhow::Result<MenuChoice> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, Hide)?;

    let mut index: usize = 0;
    let items = ["Run backup", "Exit"];

    loop {
        let (cols, _) = terminal::size()?;
        let cols = cols as usize;

        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        queue!(stdout, Print("GitHub Saver\r\n"))?;

        let description_lines = [
            "This script backs up your GitHub repositories into ./github_saves.",
            "It requires a GitHub API token (do not use a fine-grained token).",
            "The token is used only in memory and is never saved anywhere.",
            "",
        ];

        for line in description_lines {
            let mut text = line.to_string();
            if text.len() > cols {
                text.truncate(cols.saturating_sub(3));
                text.push_str("...");
            }
            queue!(stdout, Print(text), Print("\r\n"))?;
        }

        queue!(stdout, Print("\r\n"))?;

        for (i, item) in items.iter().enumerate() {
            let prefix = if i == index { ">" } else { " " };
            let line = format!("{prefix} {item}");
            queue!(stdout, Print(line), Print("\r\n"))?;
        }

        stdout.flush()?;

        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            if let KeyCode::Char('c') = code {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    execute!(stdout, Show)?;
                    terminal::disable_raw_mode()?;
                    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
                    std::process::exit(0);
                }
            }

            match code {
                KeyCode::Up => {
                    if index > 0 {
                        index -= 1;
                    }
                }
                KeyCode::Down => {
                    if index + 1 < items.len() {
                        index += 1;
                    }
                }
                KeyCode::Enter => {
                    break;
                }
                KeyCode::Char('c') => {}
                _ => {}
            }
        }
    }

    execute!(stdout, Show)?;
    terminal::disable_raw_mode()?;
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

    Ok(match index {
        0 => MenuChoice::RunBackup,
        _ => MenuChoice::Exit,
    })
}

fn start_backup_flow() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    println!("Enter GitHub API token.");
    println!("Do not use fine-grained tokens.");
    println!("The token will only be used in memory and will not be saved anywhere.");
    println!("Press Ctrl+C to cancel and return to the main menu.");
    print!("\n> ");
    stdout.flush()?;

    let token = match read_token_tui()? {
        Some(t) => t,
        None => return Ok(()),
    };

    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    println!("Loading repository list...");
    stdout.flush()?;

    let repos = fetch_all_repos(&token)?;

    if repos.is_empty() {
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        println!("No repositories found. Press Enter to continue.");
        stdout.flush()?;
        let mut dummy = String::new();
        io::stdin().read_line(&mut dummy)?;
        return Ok(());
    }

    let selected = select_repos_tui(&repos)?;

    if selected.is_empty() {
        return Ok(());
    }

    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    println!("Starting cloning into ./github_saves");
    stdout.flush()?;

    if Path::new("github_saves").exists() {
        std::fs::remove_dir_all("github_saves")?;
    }
    std::fs::create_dir_all("github_saves")?;

    for repo in selected {
        println!("Cloning {}...", repo.full_name);
        stdout.flush()?;
        let url = with_token_in_url(&repo.clone_url, &token);
        let status = Command::new("git")
            .arg("clone")
            .arg(&url)
            .arg(format!("github_saves/{}", repo.name))
            .status()?;
        if !status.success() {
            println!("Failed to clone {}", repo.full_name);
        }
    }

    println!();
    println!("Done. Press Enter to return to the menu.");
    stdout.flush()?;
    let mut dummy = String::new();
    io::stdin().read_line(&mut dummy)?;

    Ok(())
}

fn fetch_all_repos(token: &str) -> anyhow::Result<Vec<Repo>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("github-saver-rust-client"),
    );
    let auth_value = format!("token {}", token);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );

    let mut all = Vec::new();
    let mut page = 1;

    loop {
        let url = format!(
            "https://api.github.com/user/repos?per_page=100&page={}",
            page
        );
        let resp = client.get(&url).headers(headers.clone()).send()?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "GitHub API error: {}",
                resp.status()
            ));
        }
        let mut chunk: Vec<Repo> = resp.json()?;
        if chunk.is_empty() {
            break;
        }
        all.append(&mut chunk);
        page += 1;
    }

    Ok(all)
}

fn with_token_in_url(url: &str, token: &str) -> String {
    if let Some(rest) = url.strip_prefix("https://") {
        format!("https://{}@{}", token, rest)
    } else {
        url.to_string()
    }
}

fn select_repos_tui(repos: &[Repo]) -> anyhow::Result<Vec<Repo>> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, Hide)?;

    let mut index: usize = 0;
    let mut selected = vec![false; repos.len()];

    loop {
        draw_repo_list(&mut stdout, repos, &selected, index)?;

        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            if let KeyCode::Char('c') = code {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    selected.iter_mut().for_each(|s| *s = false);
                    break;
                }
            }

            match code {
                KeyCode::Up => {
                    if index > 0 {
                        index -= 1;
                    }
                }
                KeyCode::Down => {
                    if index + 1 < repos.len() {
                        index += 1;
                    }
                }
                KeyCode::Tab => {
                    if !repos.is_empty() {
                        selected[index] = !selected[index];
                    }
                }
                KeyCode::Char(' ') => {
                    if !repos.is_empty() {
                        selected[index] = !selected[index];
                    }
                }
                KeyCode::Enter => {
                    break;
                }
                _ => {}
            }
        }
    }

    execute!(stdout, Show)?;
    terminal::disable_raw_mode()?;
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

    let mut result = Vec::new();
    for (i, repo) in repos.iter().enumerate() {
        if selected[i] {
            result.push(repo.clone());
        }
    }

    Ok(result)
}

fn draw_repo_list<W: Write>(
    stdout: &mut W,
    repos: &[Repo],
    selected: &[bool],
    current: usize,
) -> anyhow::Result<()> {
    let (cols, _) = terminal::size()?;
    let cols = cols as usize;

    queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
    queue!(
        stdout,
        Print("Select repositories to back up (Tab to select):\r\n")
    )?;

    let controls = "Press Ctrl+C to cancel and return to the main menu.";
    let mut controls_line = controls.to_string();
    if controls_line.len() > cols {
        controls_line.truncate(cols.saturating_sub(3));
        controls_line.push_str("...");
    }
    queue!(stdout, Print(controls_line), Print("\r\n"))?;

    let warning = "Warning: existing repositories in ./github_saves will be deleted before cloning.";
    let mut warning_line = warning.to_string();
    if warning_line.len() > cols {
        warning_line.truncate(cols.saturating_sub(3));
        warning_line.push_str("...");
    }
    queue!(stdout, Print(warning_line), Print("\r\n\r\n"))?;

    let max_name_width = cols.saturating_sub(6);

    for (i, repo) in repos.iter().enumerate() {
        let cursor = if i == current { ">" } else { " " };
        let mark = if selected[i] { "[x]" } else { "[ ]" };
        let name = truncate_to_width(&repo.full_name, max_name_width);

        let mut line = format!("{cursor} {mark} {name}");
        let line_width = UnicodeWidthStr::width(line.as_str());
        if line_width > cols {
            line = truncate_to_width(&line, cols.saturating_sub(1));
        }

        queue!(stdout, Print(line), Print("\r\n"))?;

    }

    stdout.flush()?;
    Ok(())
}

fn truncate_to_width(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let mut result = String::new();
    let mut width = 0;

    for ch in s.chars() {
        let ch_str = ch.to_string();
        let ch_width = UnicodeWidthStr::width(ch_str.as_str());
        if width + ch_width > max_width {
            break;
        }
        result.push(ch);
        width += ch_width;
    }

    result
}

fn read_token_tui() -> anyhow::Result<Option<String>> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;

    let mut token = String::new();

    loop {
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            if let KeyCode::Char('c') = code {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    terminal::disable_raw_mode()?;
                    println!();
                    return Ok(None);
                }
            }

            match code {
                KeyCode::Char(c) => {
                    if !modifiers.contains(KeyModifiers::CONTROL) {
                        token.push(c);
                        print!("*");
                        stdout.flush()?;
                    }
                }
                KeyCode::Backspace => {
                    if !token.is_empty() {
                        token.pop();
                        print!("\u{8} \u{8}");
                        stdout.flush()?;
                    }
                }
                KeyCode::Enter => {
                    terminal::disable_raw_mode()?;
                    println!();
                    return Ok(Some(token));
                }
                _ => {}
            }
        }
    }
}
