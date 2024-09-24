// https://www.bkgm.com/rules.html
mod utils;
mod gdk;

use std::{any::Any, borrow::{Borrow, BorrowMut}};

use bevy::{prelude::*, render::camera::ScalingMode, sprite::Anchor, window::{PresentMode, WindowMode, WindowTheme}};
use bevy_simple_text_input::{TextInput, TextInputPlugin, TextInputSubmitEvent};

use gdk::GDK;
use utils::{global_to_player_tower_index, player_to_global_tower_index, Board, PLAYER_GUEST, PLAYER_HOST, TOWERS_COUNT,initialize};

const BAR_WIDTH : f32 = 100.;

const MENU_BACKGROUND_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);

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

fn create_button(parent:&mut ChildBuilder,id: &str,assets: ButtonAssets){        
    let button_style = Style {
        width: Val::Px(150.0),
        height: Val::Px(65.0),
        // border: UiRect::all(Val::Px(5.0)),
        // horizontally center child text
        justify_content: JustifyContent::Center,
        // vertically center child text
        align_items: AlignItems::Center,
        ..default()
    };
    

    let button: ButtonBundle = ButtonBundle {
        style: button_style,
        // border_color: BorderColor(Color::BLACK),
        // border_radius: BorderRadius::MAX,                    
        // background_color: NORMAL_BUTTON.into(),     
        image: UiImage::new(assets.normal.clone()) ,   
        ..default()
    };

    let button = QButtonBundle{
        id: Id::new(id.to_string()),
        button: button,
        assets: assets        
    };

    parent.spawn(button);
    // parent.spawn(button).with_children(|parent|{
    //     parent.spawn(button_text);
    // });    
}

fn create_text_input(parent:&mut ChildBuilder,font: Handle<Font>){
    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(200.0),
                border: UiRect::all(Val::Px(5.0)),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            background_color: Color::RED.into(),
            ..default()
        },
        TextInput {
            text_style: TextStyle {
                font: font,
                font_size: 40.,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
            ..default()
        },
    ));
}

fn build_buttons(commands: &mut Commands,host_button_assets: ButtonAssets,join_button_assets: ButtonAssets,font: Handle<Font>){    
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,                            
            justify_content: JustifyContent::Center,            
            column_gap: Val::Px(5.),
            ..default()
        },
        background_color: BackgroundColor::from(MENU_BACKGROUND_COLOR),
        ..default()
    })
    .with_children(|parent| {
        create_button(parent,"host_button", host_button_assets);
        create_button(parent,"join_button",join_button_assets);
        create_text_input(parent,font);
    });    
}

#[derive(Component,Clone)]
struct ButtonAssets{
    normal: Handle<Image>,
    hover: Handle<Image>,
    pressed: Handle<Image>
}

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

fn setup_game(mut commands: Commands,mut game: ResMut<Game>){
    draw_points(commands.borrow_mut(), game.wooden_stack_texture.clone(), game.white_stack_texture.clone());

    let board = game.board.as_ref().unwrap();
    draw_nuts(commands.borrow_mut(), game.wooden_nut_texture.clone(), game.white_nut_texture.clone(),board);    
}

fn setup_menu(mut commands: Commands,mut game: ResMut<Game>){
    build_buttons(commands.borrow_mut(),game.host_button_assets.clone().unwrap(),game.join_button_assets.clone().unwrap(),game.lato_regular_font.clone());
}

fn update_mainmenu(
    mut interaction_query: Query<
        (
            &Id,
            &Interaction,
            &ButtonAssets,
            &mut UiImage,
            // &mut BackgroundColor,
            // &mut BorderColor,
            // &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,    
) {  
    
    for (id,interaction,assets,mut image/* , mut color, mut border_color, children*/) in &mut interaction_query {
        
        match (*interaction){
            Interaction::None => {
                image.texture = assets.normal.clone();
                println!("{}","normal");
            },
            Interaction::Hovered=> {
                image.texture = assets.hover.clone();
                println!("{}","hover");
            },
            Interaction::Pressed =>{
                if id.id == "host_button"{
                    next_state.set(GameState::InGame);
                }
                if id.id == "join_button"{
                    next_state.set(GameState::InGame);
                }
                image.texture = assets.pressed.clone();
                println!("{}","pressed");
            },
            _ => ()
        }
    }    
}


#[derive(Bundle)]
struct QButtonBundle {   
    id: Id,
    button: ButtonBundle,
    assets: ButtonAssets
}

#[derive(Component)]
struct Id{
    pub id: String,
}

impl  Id {
    fn new(value: String)->Id{
        return Id { id: value }
    }
}

fn input_listener(mut events: EventReader<TextInputSubmitEvent>) {
    for event in events.read() {
        info!("{:?} submitted: {}", event.entity, event.value);
    }
}


fn create_game(mut backend: ResMut<Backend>){
    let future = backend.gdk.start_game();    
}


#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState{    
    #[default]
    MainMenu,
    CreatingGame,
    JoiningGame,    
    InGame
}

// remove all entities that are not a camera or window
fn tear_down(mut commands: Commands, entities: Query<Entity, (Without<Camera>, Without<Window>)>) {
    for entity in &entities {
        commands.entity(entity).despawn();     
    }
}

#[derive(Resource, Default)]
struct Game{    
    board:Option<Board>,
    wooden_stack_texture: Handle<Image>,
    white_stack_texture: Handle<Image>,
    wooden_nut_texture: Handle<Image>,
    white_nut_texture: Handle<Image>,
    host_button_assets :Option<ButtonAssets>,
    join_button_assets: Option<ButtonAssets>,    
    lato_regular_font: Handle<Font>
}

#[derive(Resource)]
struct Backend{
    gdk: GDK
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