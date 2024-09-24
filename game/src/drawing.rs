
use std::{any::Any, borrow::{Borrow, BorrowMut}};

use bevy::{prelude::*, render::camera::ScalingMode, sprite::Anchor, window::{PresentMode, WindowMode, WindowTheme}};
use bevy_simple_text_input::{TextInput, TextInputPlugin, TextInputSubmitEvent};

use crate::utils::{global_to_player_tower_index, player_to_global_tower_index, Board, PLAYER_GUEST, PLAYER_HOST, TOWERS_COUNT,initialize};
use crate::resources::{Backend,Game};

const BAR_WIDTH : f32 = 100.;


pub fn setup_game(mut commands: Commands,mut game: ResMut<Game>){
    draw_points(commands.borrow_mut(), game.wooden_stack_texture.clone(), game.white_stack_texture.clone());

    let board = game.board.as_ref().unwrap();
    draw_nuts(commands.borrow_mut(), game.wooden_nut_texture.clone(), game.white_nut_texture.clone(),board);    
}

fn draw_points(commands:&mut Commands,wooden_stack_texture: Handle<Image>,white_stack_texture: Handle<Image>){
    //Render top    

    for i in 0..6{
        let texture_handle: Handle<Image>;

        if i % 2 == 1 {
            texture_handle = wooden_stack_texture.clone();
        } else{
            texture_handle = white_stack_texture.clone();
        }
        let mut sprite = SpriteBundle{
            texture: texture_handle ,
            transform: Transform::from_xyz(-50. +(i as f32) * -75.0 , 0.,-0.1),            
            ..default()
        };
        sprite.sprite.anchor  = Anchor::BottomCenter;
        commands.spawn(sprite);
    }

    for i in 0..6{
        let texture_handle: Handle<Image>;

        if i % 2 == 1 {
            texture_handle = wooden_stack_texture.clone();
        } else{
            texture_handle = white_stack_texture.clone();
        }
        let mut sprite = SpriteBundle{
            texture: texture_handle ,
            transform: Transform::from_xyz(50. +(i as f32) * 75.0 , 0., -0.1),            
            ..default()
        };
        sprite.sprite.anchor  = Anchor::BottomCenter;
        commands.spawn(sprite);
    }
    

    //Render bottom

    for i in 0..6{
        let texture_handle: Handle<Image>;

        if i % 2 == 1 {
            texture_handle = wooden_stack_texture.clone();
        } else{
            texture_handle = white_stack_texture.clone();
        }
        let sprite = SpriteBundle{
            texture: texture_handle ,
            sprite: Sprite { 
                anchor: Anchor::TopCenter,              
                flip_y: true,  
                ..default()
            },
            transform: Transform::from_xyz(-50. +(i as f32) * -75.0 , 720., -0.1),            
            ..default()
        };
        commands.spawn(sprite);
    }

    for i in 0..6{
        let texture_handle: Handle<Image>;

        if i % 2 == 1 {
            texture_handle = wooden_stack_texture.clone();
        } else{
            texture_handle = white_stack_texture.clone();
        }
        let sprite = SpriteBundle{
            texture: texture_handle ,
            sprite: Sprite { 
                anchor: Anchor::TopCenter,   
                flip_y: true,             
                ..default()
            },
            transform: Transform::from_xyz(50. +(i as f32) * 75.0 , 720., -0.1),            
            ..default()
        };
        commands.spawn(sprite);
    }
}


fn get_nut_position(global_tower_index: usize,nut_index: usize ) -> Vec3{
    let mut local_index = global_tower_index % 6;
    let is_right = (global_tower_index  / 6) % 2;
    let dir = ((global_tower_index / 12) as f32 - 1.) * -1.;
    println!("{}",local_index);
    if is_right == 0 {
        local_index = 5 - local_index;
    }    
    if global_tower_index < 6 {
        Vec3::from_array([ (BAR_WIDTH * 0.5) + local_index as f32 * 75.,nut_index as f32 * dir * 65. - 10.,0.])    
    }else if global_tower_index < 12 {
        Vec3::from_array([ -(BAR_WIDTH * 0.5) + local_index as f32 * -75.,nut_index as f32 * dir * 65. - 10.,0.])    
    }else if global_tower_index < 18 {
        Vec3::from_array([ -(BAR_WIDTH * 0.5) + local_index as f32 * -75.,720. - 85. - (nut_index as f32  * 65.),0.])    
    }else{
        Vec3::from_array([ (BAR_WIDTH * 0.5) + local_index as f32 * 75.,720. - 85. - (nut_index as f32 * 65.),0.])    
    }    
}


fn draw_nuts(commands: &mut Commands,wooden_nut_texture: Handle<Image>,white_nut_texture: Handle<Image> , board: &Board){
    
    for tower_id in 1..TOWERS_COUNT+1{
        let texture_handle: Handle<Image>;                        
        let tower = board.towers[tower_id-1];
        if tower.owner == PLAYER_HOST as u8 {
            texture_handle = wooden_nut_texture.clone();
        } else{
            texture_handle = white_nut_texture.clone();
        }

        for nut_index in 0..tower.nuts{
            let sprite = SpriteBundle{
                texture: texture_handle.clone() ,
                transform: Transform::from_translation(get_nut_position(tower_id - 1, nut_index as usize)),
                sprite: Sprite{
                    anchor : Anchor::BottomCenter,
                    ..default()
                },
                ..default()
            };
            
            commands.spawn(sprite);
        }
        
    }
        
}
