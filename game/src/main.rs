// https://www.bkgm.com/rules.html

use std::borrow::{Borrow, BorrowMut};

use bevy::{prelude::*, render::camera::ScalingMode, sprite::Anchor, window::{PresentMode, WindowMode, WindowTheme}};

#[path = "./utils.rs"]
mod utils;
use utils::{global_to_player_tower_index, player_to_global_tower_index, Board, PLAYER_GUEST, PLAYER_HOST, TOWERS_COUNT,initialize};

const BAR_WIDTH : f32 = 100.;

fn main() {    
    App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Aptos Backgammon".into(),
            name: Some("backgammon.app".into()),
            // mode: WindowMode::BorderlessFullscreen,
            resolution: (1280., 720.).into(),
            present_mode: PresentMode::AutoVsync,
            // Tells Wasm to resize the window according to the available canvas
            fit_canvas_to_parent: true,
            // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
            prevent_default_event_handling: false,
            window_theme: Some(WindowTheme::Dark),
            enabled_buttons: bevy::window::EnabledButtons {
                maximize: false,
                ..Default::default()
            },
            // This will spawn an invisible window
            // The window will be made visible in the make_visible() system after 3 frames.
            // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
            visible: true,
            ..default()
        }),
        ..default()
    }),)
    .add_systems(Startup, setup)
    .run();    
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

fn setup(mut commands: Commands,asset_server: Res<AssetServer>){
    let wooden_stack_texture = asset_server.load("wooden_stack.png");    
    let white_stack_texture = asset_server.load("white_stack.png");        
    let wooden_nut_texture = asset_server.load("wooden_nut.png");    
    let white_nut_texture = asset_server.load("white_nut.png");    

    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedVertical(720.);
    camera.transform = Transform::from_xyz(0., 360., 0.0);    

    commands.spawn(camera);

    let board = initialize();    

    draw_points(commands.borrow_mut(), wooden_stack_texture, white_stack_texture);
    draw_nuts(commands.borrow_mut(), wooden_nut_texture, white_nut_texture,&board);
}
