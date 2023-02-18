use othello_core_lib::*;
use crate::{ai::*, game::*};
use std::{
    error::Error,
    io,
};


#[derive(Debug)]
pub enum MixedPlayer {
    AI(AI),
    Human,
}

impl MixedPlayer {
    pub fn try_clone(&self) -> Result<Self, Box<dyn Error>> {
        match self {
            MixedPlayer::AI(ai) => Ok(MixedPlayer::AI(ai.try_clone()?)),
            MixedPlayer::Human => Ok(MixedPlayer::Human),
        }
    }
}

impl Player for MixedPlayer {
    fn name(&self) -> String {
        match self {
            MixedPlayer::AI(ai) => ai.name(),
            MixedPlayer::Human => "human".to_owned(),
        }
    }

    fn init(&mut self,pos:Pos) -> io::Result<()>  {
        match self {
            MixedPlayer::AI(ai) => ai.init(pos),
            MixedPlayer::Human => Ok(()),
        }
    }

    fn update(&mut self,pos:Pos) -> io::Result<UpdateResult>  {
        match self {
            MixedPlayer::AI(ai) => ai.update(pos),
            MixedPlayer::Human =>  Ok(UpdateResult::Wait),
        }
    }

    fn interrupt(&mut self) -> io::Result<()>  {
        match self {
            MixedPlayer::AI(ai) => ai.interrupt(),
            MixedPlayer::Human => Ok(()),
        }
    }
}
