#[allow(deprecated)]
use std::env::home_dir;
use std::{fs, io};
use std::path::Path;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize, Text, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, BorderType, Paragraph};
use crate::window_managers::WindowManager;

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    window_managers: Vec<WindowManager>,
    selected: usize,
}

impl App {
    pub fn new() -> Self {
        let mut res = Self::default();
        #[allow(deprecated)]
        let home = match home_dir() {
            Some(path) => path,
            None => {
                eprintln!("There was an error retrieving the home directory.\nNo window managers will be used, will also auto-exit on run until fixed");
                return res;
            },
        };

        if !home.exists() {
            eprintln!("Home directory doesn't exist.\nNo window managers will be used, will also auto-exit on run until fixed");
            return res;
        }

        let home = match home.to_str() {
            Some(path) => path,
            None => {
                eprintln!("There was an error converting home PathBuf to a &str.\nNo window managers will be used, will also auto-exit on run until fixed");
                return res;
            }
        };

        let config_folder = if home.ends_with("/") { home.to_owned() + ".config/wmtui" } else { home.to_owned() + "/.config/wmtui" };

        if !Path::new(&config_folder).exists() {
            println!("~/.config/wmtui folder doesn't exist, creating. Don't forget to create a window_managers.json file");
            match fs::create_dir_all(&config_folder) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("There was an error creating the config folder.\n\n{e:?}\n\nExiting...");
                    std::process::exit(1);
                }
            }
	    std::process::exit(2);
        }

        let window_managers_file = match fs::File::open(config_folder + "/window_managers.json") {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error opening window_managers file: {e:?}");
                std::process::exit(1);
            }
        };

        let window_managers: Vec<WindowManager> = serde_json::from_reader(window_managers_file).unwrap_or_else(|e| {
            eprintln!("Error parsing window_managers file: {e:?}");
            std::process::exit(1);
        });

        res.window_managers = window_managers;

        res
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        if self.window_managers.is_empty() {
            return Ok(());
        }
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                self.exit();
                self.selected = self.window_managers.len();
            },
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            KeyCode::Enter => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        if self.selected >= self.window_managers.len() - 1 {
            self.selected = 0;
        } else {
            self.selected += 1;
        }
    }

    fn decrement_counter(&mut self) {
        if self.selected == 0 || self.selected >= self.window_managers.len() {
            self.selected = self.window_managers.len().saturating_sub(1);
        } else {
            self.selected = self.selected - 1;
        }
    }

    pub fn select(&mut self) -> Option<String> {
        self.exit = true;
        self.window_managers.get(self.selected).map(|wm| wm.exec.clone())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" WMTUI ".bold());
        let instructions = Line::from(vec![
            " Cycle Left ".into(),
            "<Left>".blue().bold(),
            " Cycle Right ".into(),
            "<Right>".blue().bold(),
            " Select ".into(),
            "<Enter>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK)
            .border_type(BorderType::Plain);

        let counter_text = Text::from(vec![Line::from(vec![
            "Window Manager: ".into(),
            self.window_managers.get(self.selected).map(|wm| wm.name.clone()).unwrap_or(String::from("Something went wrong with the counter... whoopsss...")).yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
