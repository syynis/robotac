use itertools::Itertools;
use ratatui::{
    crossterm::event::{Event, KeyCode},
    prelude::*,
};

use crate::{app::Message, popup::Popup};

pub struct SaveHistory {
    input: String,
}

impl Default for SaveHistory {
    fn default() -> Self {
        Self {
            input: String::default(),
        }
    }
}

impl SaveHistory {
    pub fn update(&mut self, event: &Event) -> Option<Message> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char(c) => {
                    if c.is_ascii() {
                        self.input.push(c)
                    }
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Enter => {
                    return Some(Message::SaveHistory(self.input.clone()));
                }
                _ => {}
            }
        }
        None
    }
    pub fn draw(&self) -> impl Widget + '_ {
        Popup::default()
            .title("Input name for history".to_string())
            .content(self.input.clone())
    }
}

pub struct LoadHistory {
    selected: usize,
}

impl Default for LoadHistory {
    fn default() -> Self {
        Self { selected: 0 }
    }
}

impl LoadHistory {
    pub fn update(&mut self, event: &Event) -> Option<Message> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Right | KeyCode::Char('j') => {
                    let file_count = std::fs::read_dir("histories")
                        .unwrap()
                        .filter(|s| {
                            if let Some(s) = s.as_ref().ok() {
                                let path = s.path();
                                !path.is_dir()
                            } else {
                                false
                            }
                        })
                        .count();
                    self.selected = (self.selected + 1).min(file_count - 1);
                }
                KeyCode::Left | KeyCode::Char('k') => {
                    self.selected = self.selected.saturating_sub(1);
                }
                KeyCode::Enter => {
                    let file = &std::fs::read_dir("histories")
                        .unwrap()
                        .filter_map(|s| {
                            let s = s.ok()?;
                            let path = s.path();
                            if !path.is_dir() {
                                Some(s)
                            } else {
                                None
                            }
                        })
                        .collect_vec()[self.selected];
                    let name = file.file_name().to_str().unwrap().to_owned();
                    return Some(Message::LoadHistory(name));
                }
                _ => {}
            }
        }
        None
    }
    pub fn draw(&self) -> impl Widget + '_ {
        let files = std::fs::read_dir("histories")
            .unwrap()
            .filter_map(|s| {
                let s = s.ok()?;
                let path = s.path();
                if !path.is_dir() {
                    Some(s.file_name().to_str().unwrap().to_owned())
                } else {
                    None
                }
            })
            .enumerate()
            .map(|(idx, s)| {
                if idx == self.selected {
                    format!(">{}", s)
                } else {
                    s
                }
            })
            .join("\n");
        Popup::default()
            .title("Histories".to_owned())
            .content(files)
    }
}
