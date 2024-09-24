use bevy::{prelude::*, render::camera::ScalingMode, sprite::Anchor, window::{PresentMode, WindowMode, WindowTheme}};
use crate::utils::Board;
use crate::components::ButtonAssets;
use crate::gdk::GDK;


#[derive(Resource, Default)]
pub struct Game{    
    pub board:Option<Board>,
    pub wooden_stack_texture: Handle<Image>,
    pub white_stack_texture: Handle<Image>,
    pub wooden_nut_texture: Handle<Image>,
    pub white_nut_texture: Handle<Image>,
    pub host_button_assets :Option<ButtonAssets>,
    pub join_button_assets: Option<ButtonAssets>,    
    pub lato_regular_font: Handle<Font>
}

#[derive(Resource)]
pub struct Backend{
    pub gdk: GDK
}