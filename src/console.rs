use std::io::stdout;
use crossterm::{cursor, ExecutableCommand, terminal};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Console {
    pinned: Option<String>,
    pub level: Level,
}

impl Console {
    pub fn print_with_level(&self, level: Level, message: &str) {
        #[cfg(not(debug_assertions))]
        if level == Level::Debug {
            return;
        }

        if level < self.level {
            return;
        }

        self.clear_pinned();

        println!("{message}");

        if let Some(pinned) = &self.pinned {
            print!("{}", pinned);
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
        self.clear_pinned();

        print!("{pinned}");
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
    // debug is only printed in debug builds
    Debug = 0,
}

