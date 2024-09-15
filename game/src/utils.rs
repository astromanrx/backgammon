
//// GAME CONSTANTS    
pub const TOWERS_COUNT: usize = 24;

//// PLAYER CONSTANTS
pub const PLAYER_HOST: usize = 0;
pub const PLAYER_GUEST: usize = 1;

#[derive(Copy, Clone)]
pub struct Tower {
    pub nuts: u8,
    pub owner: u8
}

pub struct Bar {
    host_nuts: u8,
    guest_nuts: u8
}

//TODO: simplify the tower to (u8,u8) for player index and num of the player's nuts in the tower
pub struct Board {
    pub towers: [Tower;TOWERS_COUNT],  
    pub bar: Bar,		
}	

impl Board{
    
}

pub fn initialize() -> Board{
    
    let bar = Bar{
        guest_nuts: 0,
        host_nuts: 0
    };
    let mut towers:[Tower;TOWERS_COUNT] = [Tower{nuts:0,owner:PLAYER_GUEST as u8};TOWERS_COUNT];

    let tower_ids = [1,12,17,19];
    let tower_nuts = [2,5,3,5];

    for player in [PLAYER_HOST,PLAYER_GUEST]{            
        for i in 0..4{
            
            let tower_id = tower_ids[i];
            let global_index = player_to_global_tower_index(player, tower_id - 1);
            towers[global_index] = Tower{
                nuts:tower_nuts[i],
                owner: player as u8
            };
        }
    }
    let board = Board{
        bar: bar,
        towers: towers
    };

    board
}

pub fn player_to_global_tower_index(player:usize, tower_index: usize)-> usize{
    if player == PLAYER_HOST {
        tower_index
    }else{
        TOWERS_COUNT - tower_index - 1
    }        
}

pub fn global_to_player_tower_index(player:usize, tower_index: usize)-> usize{
    if player == PLAYER_HOST {
        TOWERS_COUNT - tower_index - 1
    }else{
        tower_index
    }            
}

