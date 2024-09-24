// https://www.bkgm.com/rules.html
mod utils;
mod gdk;
mod drawing;
mod ui;
mod resources;
mod states;
mod components;

use std::{any::Any, borrow::{Borrow, BorrowMut}};

use bevy::{prelude::*, render::camera::ScalingMode, sprite::Anchor, window::{PresentMode, WindowMode, WindowTheme}};
use bevy_simple_text_input::{TextInput, TextInputPlugin, TextInputSubmitEvent};

use crate::gdk::GDK;
use crate::utils::{global_to_player_tower_index, player_to_global_tower_index, Board, PLAYER_GUEST, PLAYER_HOST, TOWERS_COUNT,initialize};

use crate::drawing::setup_game;
use crate::ui::{setup_menu,update_mainmenu,input_listener};
use crate::resources::{Backend,Game};
use crate::components::{ButtonAssets,Id};
use crate::states::GameState;


fn load_assets(mut commands: Commands,asset_server: Res<AssetServer>,mut game: ResMut<Game>){
    
    let wooden_stack_texture: Handle<Image> = asset_server.load("sprites/game/wooden_stack.png");    
    let white_stack_texture: Handle<Image> = asset_server.load("sprites/game/white_stack.png");        
    let wooden_nut_texture: Handle<Image> = asset_server.load("sprites/game/wooden_nut.png");    
    let white_nut_texture: Handle<Image> = asset_server.load("sprites/game/white_nut.png");        

    let host_button_assets = ButtonAssets{
        normal: asset_server.load("sprites/ui/host_button/normal.png"),
        hover: asset_server.load("sprites/ui/host_button/hover.png"),
        pressed: asset_server.load("sprites/ui/host_button/pressed.png")
    };
    let join_button_assets = ButtonAssets{
        normal: asset_server.load("sprites/ui/join_button/normal.png"),
        hover: asset_server.load("sprites/ui/join_button/hover.png"),
        pressed: asset_server.load("sprites/ui/join_button/pressed.png")
    };
    
    let lato_regular_font: Handle<Font> = asset_server.load("fonts/Lato/Lato-Regular.ttf");    

    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedVertical(720.);
    camera.transform = Transform::from_xyz(0., 360., 0.0);    

    commands.spawn(camera);

    let board = initialize();        

    game.board =Some(board);
    game.host_button_assets = Some(host_button_assets);
    game.join_button_assets = Some(join_button_assets);
    game.lato_regular_font = lato_regular_font;
    game.wooden_stack_texture = wooden_stack_texture;
    game.wooden_nut_texture = wooden_nut_texture;
    game.white_stack_texture = white_stack_texture;
    game.white_nut_texture = white_nut_texture;
}




fn create_game(mut backend: ResMut<Backend>){
    let future = backend.gdk.start_game();    
}


// remove all entities that are not a camera or window
fn tear_down(mut commands: Commands, entities: Query<Entity, (Without<Camera>, Without<Window>)>) {
    for entity in &entities {
        commands.entity(entity).despawn();     
    }
}



#[tokio::main]
async fn main() {    
    let gdk = GDK::new();        
    let wallet_address = gdk.get_address();
    // gdk.start_game().await;
    

    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Aptos Backgammon".into(),
            // name: Some("backgammon.app".into()),
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
    .init_resource::<Game>()
    .insert_resource(Backend{
        gdk: gdk
    })
    .add_state::<GameState>()    
    .add_plugins(TextInputPlugin)
    .add_systems(Startup, load_assets)
    .add_systems(OnEnter(GameState::InGame), setup_game.after(load_assets))
    .add_systems(OnEnter(GameState::MainMenu), setup_menu.after(load_assets))
    .add_systems(Update, update_mainmenu.run_if(in_state(GameState::MainMenu)))
    .add_systems(Update, input_listener.run_if(in_state(GameState::MainMenu)))
    .add_systems(OnExit(GameState::InGame),tear_down)
    .add_systems(OnExit(GameState::MainMenu),tear_down)
    ;
    // app.add_state<State>();     

    // app.configure_sets(Update, input_listener.run_if(in_state(State::InGame)) );    
    app.run();   
    
}