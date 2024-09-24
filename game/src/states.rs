use bevy::{prelude::*, render::camera::ScalingMode, sprite::Anchor, window::{PresentMode, WindowMode, WindowTheme}};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState{    
    #[default]
    MainMenu,
    CreatingGame,
    JoiningGame,    
    InGame
}