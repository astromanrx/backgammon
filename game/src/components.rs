use bevy::prelude::*;


#[derive(Component,Clone)]
pub struct ButtonAssets{
    pub normal: Handle<Image>,
    pub hover: Handle<Image>,
    pub pressed: Handle<Image>
}

#[derive(Component)]
pub struct Id{
    pub id: String,
}

impl  Id {
    pub fn new(value: String)->Id{
        return Id { id: value }
    }
}
