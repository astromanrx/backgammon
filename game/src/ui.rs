
use std::{any::Any, borrow::{Borrow, BorrowMut}};

use bevy::{prelude::*, render::camera::ScalingMode, sprite::Anchor, window::{PresentMode, WindowMode, WindowTheme}};
use bevy_simple_text_input::{TextInput, TextInputPlugin, TextInputSubmitEvent};

use crate::gdk::GDK;
use crate::resources::{Backend,Game};
use crate::states::GameState;
use crate::components::*;


const MENU_BACKGROUND_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);

#[derive(Bundle)]
struct QButtonBundle {   
    id: Id,
    button: ButtonBundle,
    assets: ButtonAssets
}


pub fn setup_menu(mut commands: Commands,mut game: ResMut<Game>){
    build_buttons(commands.borrow_mut(),game.host_button_assets.clone().unwrap(),game.join_button_assets.clone().unwrap(),game.lato_regular_font.clone());
}

pub fn input_listener(mut events: EventReader<TextInputSubmitEvent>) {
    for event in events.read() {
        info!("{:?} submitted: {}", event.entity, event.value);
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

pub fn update_mainmenu(
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
            },
            Interaction::Hovered=> {
                image.texture = assets.hover.clone();
            },
            Interaction::Pressed =>{
                if id.id == "host_button"{
                    next_state.set(GameState::CreatingGame);
                    println!("Host button pressed");
                }
                if id.id == "join_button"{
                    next_state.set(GameState::JoiningGame);
                    println!("Join button pressed");
                }
                image.texture = assets.pressed.clone();
                
            },
            _ => ()
        }
    }    
}

