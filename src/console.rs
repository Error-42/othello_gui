use std::io::{Write, stdout};
use crossterm::{cursor, ExecutableCommand, terminal};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Console {
    pinned: Option<String>,
    pub level: Level,
}

impl Console {
    pub fn new(level: Level) -> Self {
        Self {
            pinned: None,
            level,
        }
    }

    pub fn print_with_level(&self, level: Level, message: &str) {
        if level < self.level || (cfg!(debug_assert) && level == Level::Debug) {
            return;
        }

        self.clear_pinned();

        println!("{message}");

        if let Some(pinned) = &self.pinned {
            print!("{}", pinned);
            stdout().flush().unwrap();
        }
    }

    pub fn print(&self, message: &str) {
        self.print_with_level(Level::Necessary, message);
    }

    pub fn warn(&self, message: &str) {
        self.print_with_level(Level::Warning, message);
    }

    pub fn info(&self, message: &str) {
        self.print_with_level(Level::Info, message);
    }

    pub fn debug(&self, message: &str) {
        self.print_with_level(Level::Debug, message);
    }

    pub fn pin(&mut self, pinned: String) {
        if let Some(already_pinned) = &self.pinned {
            if *already_pinned == pinned {
                return;
            }
        }

        self.clear_pinned();

        print!("{pinned}");
        stdout().flush().unwrap();
        self.pinned = Some(pinned);
    }

    pub fn unpin(&mut self) {
        self.clear_pinned();

        self.pinned = None;
    }

    fn clear_pinned(&self) {
        if let Some(_) = self.pinned {
            stdout()
                .execute(terminal::Clear(terminal::ClearType::CurrentLine))
                .unwrap()
                .execute(cursor::MoveToColumn(0))
                .unwrap();
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Necessary = 3,
    Warning = 2,
    Info = 1,
    // debug is printed only and always in debug builds
    Debug = 0,
}

