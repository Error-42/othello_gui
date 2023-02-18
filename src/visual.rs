use crate::{console::*, game::*, mixed_player::*};


#[derive(Debug)]
pub struct Visual {
    pub game: Game<MixedPlayer>,
    pub console: Console,
}

impl Visual {
    pub fn undo(&mut self) {
        if let [MixedPlayer::AI(_), MixedPlayer::AI(_)] = self.game.players {
            return;
        }
    
        self.game.manual_interrupt(&self.console);
        
        while self.game.history.len() >= 2 {
            self.game.manual_undo(&self.console);
    
            if let Some(MixedPlayer::Human) = self.game.next_player() {
                break;
            }
        }
    
        self.game.initialize_next_player(&self.console);
    }
}

impl Showable for Visual {
    fn display_pos(&self) -> DisplayPos {
        self.game.display_pos()
    }
}

