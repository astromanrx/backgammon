// Game rules reference: https://www.bkgm.com/rules.html

module backgammon::backgammon {
    // use aptos_framework::account;
    use aptos_framework::event;
    use std::error;
    use std::option::{Self, Option};
    use std::signer;
    use std::vector;
	use aptos_framework::randomness;

    //// GAME CONSTANTS    
    const TOWERS_COUNT: u8 = 24;
    //// PLAYER CONSTANTS
    const PLAYER_HOST: u8 = 0;
    const PLAYER_GUEST: u8 = 1;

    //// ERROR CODES
    /// Placing a move on a (x,y) position that is already occupied
    const EINVALID_MOVE: u64 = 0;
    /// An address trying to claim a player that is already taken
    const EPLAYER_TAKEN: u64 = 1;
    /// Trying to destroy game before game has been finished
    const EGAME_NOT_DONE: u64 = 2;
    /// An address can only join as one player, not two
    const ECANNOT_JOIN_AS_TWO_PLAYERS: u64 = 3;
    /// A (x,y) move is out of bounds of the 3x3 grid
    const EOUT_OF_BOUNDS_MOVE: u64 = 4;
    /// Address doesn't exist in the current Game
    const EPLAYER_NOT_IN_GAME: u64 = 5;
    /// An attempt to place moves even though a Game is over
    const EGAME_HAS_ALREADY_FINISHED: u64 = 6;
    /// User tries to make two games
    const EGAME_ALREADY_EXISTS_FOR_USER: u64 = 7;
    /// Game doesn't exist under address
    const EGAME_DOESNT_EXIST: u64 = 8;
    /// Out of turn move or player not in game
    const EOUT_OF_TURN_MOVE: u64 = 9;
    // Has nuts out of home , can't bear off
    const EBEAR_OFF_ERROR_NUTS_OUT_OF_HOME: u64 = 10;
    // Dice index is invalid
    const EINVALID_DICE_INDEX: u64 = 11;
    // Tower index is invalid
    const EINVALID_TOWER_INDEX: u64 = 12;
    // Dice number is invalid
    const EINVALID_DICE_NUM: u64 = 13;
    // Player trying to change other player's tower
    const EINVALID_TOWER_PLAYER: u64 = 14;
    // It's not your turn
    const EPLAYER_NOT_YOUR_TURN: u64 = 15;


    #[event]
    struct GameOverEvent has drop, store {
        game_address: address,
        is_game_over: bool,
    }

    #[event]
    struct EndOfTurnEvent has drop, store {
        game_address: address
    }

    struct Player has copy, drop, store {
        type: u8,
        owner: address,
    }
    

    struct Tower has drop,store{
        nuts: u8,
        owner: u8
    }

    struct Bar has drop,store{
        host_nuts: u8,
        guest_nuts: u8
    }

    //TODO: simplify the tower to (u8,u8) for player index and num of the player's nuts in the tower
    struct Board has drop, store {
        towers: vector<Tower>,  
		bar: Bar,		
    }	

    //// to access game records after games are over.
    struct Game has key, store {
        board: Board,
		host_dices:vector<u8>,
        guest_dices:vector<u8>,
        host_player: Option<Player>,
        guest_player: Option<Player>,
        active_player: u8,
        is_game_over: bool,
    }

    /*
     * @notice initializes a valid, playable Game
     * @dev stores the Game into global storage
     */
    //// TODO: have Game as its own object, with its own address
    public entry fun create_game(creator: &signer) {
        // check game doesn't already exist under creator address
        assert!(!exists<Game>(signer::address_of(creator)), error::already_exists(EGAME_ALREADY_EXISTS_FOR_USER));
        let game = initalize_game();
        let creator_addr = signer::address_of(creator);
        choose_host_player(&mut game, creator_addr);
        move_to<Game>(creator, game);                
    }

    /*
     * @notice lets another user join given a valid game address
     */
    public entry fun join_game(new_user: &signer, game_addr: address) acquires Game {
        assert!(exists<Game>(game_addr), error::not_found(EGAME_DOESNT_EXIST));

        let new_user_addr = signer::address_of(new_user);
        assert!(new_user_addr != game_addr, error::invalid_argument(ECANNOT_JOIN_AS_TWO_PLAYERS));
        
        let game = borrow_global_mut(game_addr);
        choose_guest_player(game, new_user_addr);
    }

    /*
     * @notice places a move at a given (x,y) coordinate on a 3x3 board
     * @dev checks to ensure a player can make a valid move
     */
    public entry fun choose_move(player: &signer, game_addr: address, tower_index: u8, dice_index: u8) acquires Game {        
        assert!(exists<Game>(game_addr), error::not_found(EGAME_DOESNT_EXIST));
        let game: &mut Game = borrow_global_mut(game_addr);        
        let host_player = option::borrow(&game.host_player);
        let guest_player = option::borrow(&game.guest_player);                    
        let player_addr = signer::address_of(player);
        
        assert!(
            player_addr != host_player.owner || player_addr != guest_player.owner,
            error::permission_denied(EPLAYER_NOT_IN_GAME),
        );        
        
        assert!(game.active_player == PLAYER_GUEST && (player_addr != guest_player.owner) , error::permission_denied(EPLAYER_NOT_YOUR_TURN));
        assert!(game.active_player == PLAYER_HOST && (player_addr != host_player.owner) , error::permission_denied(EPLAYER_NOT_YOUR_TURN));

        place_move(game, tower_index, dice_index);        

        let dices: &mut vector<u8>;

        if(game.active_player == PLAYER_HOST){
            dices = &mut game.host_dices;
        }else{
            dices = &mut game.guest_dices;
        };

        vector::remove(dices,dice_index as u64); 

        if(vector::length(dices) == 0){
            event::emit(EndOfTurnEvent { game_address: game_addr, });
        }
            
    }

    #[view]
    public fun get_dices(player_addr: address, game_addr: address) : vector<u8> acquires Game{          
        assert!(exists<Game>(game_addr), error::not_found(EGAME_DOESNT_EXIST));
  
        let game: &Game = borrow_global(game_addr);
        let host_player = option::borrow(&game.host_player);
        let guest_player = option::borrow(&game.guest_player);
        assert!(host_player.owner == player_addr || guest_player.owner == player_addr , error::permission_denied(EPLAYER_NOT_YOUR_TURN));
        if(host_player.owner == player_addr){
            return game.host_dices
        }else{
            return game.guest_dices
        }
    }   

    /*
     * @notice destroy Game at the end of session / forfeit
     */
    // public entry fun cleanup(creator: &signer) acquires Game {
    //     let creator_addr: address = signer::address_of(creator);
    //     // abort if no such game exists under creator
    //     let game: Game = move_from<Game>(creator_addr);
    //     // cleanup_game(game);
    // }

    /*
     * @notice voluntarily give up, the other player wins
     */
    public entry fun forfeit(player: &signer, game_addr: address) acquires Game {
        assert!(exists<Game>(game_addr), error::not_found(EGAME_DOESNT_EXIST));

        let player_addr = signer::address_of(player);
        let game: &mut Game = borrow_global_mut(game_addr);
        let host_player = option::borrow_mut(&mut game.host_player);
        let guest_player = option::borrow_mut(&mut game.guest_player);

        assert!(
            player_addr != host_player.owner || player_addr != guest_player.owner,
            error::permission_denied(EPLAYER_NOT_IN_GAME)
        );
		
		//TODO: must determine player that requested the forfeit and set it as the loser

        game.is_game_over = true;

        event::emit(GameOverEvent { game_address: game_addr, is_game_over: true, });
    }
	
	fun player_to_global_tower_index(player:u8, tower_index: u8): u8{
		if(player == PLAYER_HOST)
			tower_index
		else
			TOWERS_COUNT - tower_index - 1
		
	}

    fun global_to_player_tower_index(player:u8, tower_index: u8): u8{
		if(player == PLAYER_HOST)
			TOWERS_COUNT - tower_index - 1
		else
			tower_index
		
	}

	fun push_nut(game: &mut Game,player: u8,player_tower_index:u8 , count: u8){
		let global_tower_index = player_to_global_tower_index(player,player_tower_index);
        let tower = vector::borrow_mut<Tower>(&mut game.board.towers,global_tower_index as u64);
        tower.nuts = tower.nuts + count;
        tower.owner = player;		    
	}
		

    /*
     * @notice initialize Game struct with base values 
     */
    fun initalize_game(): Game {		
        let towers: vector<Tower> = vector::empty();
		let bar: Bar;
		
		for (i in 0..TOWERS_COUNT){            
            vector::push_back(&mut towers,Tower{
                owner:0,
                nuts:0
            });		
        };
			
		
		bar = Bar{
            host_nuts: 0,
            guest_nuts: 0,
        };
        
        let game = Game {
            board: Board {
                towers: towers,                
				bar: bar
            },
            host_player: option::none(),
            guest_player: option::none(),
            
            host_dices: vector::empty(),
            guest_dices: vector::empty(),
            
            active_player: PLAYER_HOST,
            is_game_over: false,

        };
		
		for(player in PLAYER_HOST..PLAYER_GUEST){            
            push_nut(&mut game,player,0,2);
            push_nut(&mut game,player,11,5);            
            push_nut(&mut game,player,16,3);            
            push_nut(&mut game,player,18,5);
        };
			
		game
    }
	
	#[randomness(max_gas=56789)]
    entry fun roll_the_dice(player: signer,game_addr: address) acquires Game  {
        assert!(exists<Game>(game_addr), error::not_found(EGAME_DOESNT_EXIST));

        let game: &mut Game = borrow_global_mut(game_addr);

		let player_addr = signer::address_of(&player);
        let host_player = option::borrow_mut(&mut game.host_player);
        let guest_player = option::borrow_mut(&mut game.guest_player);
        assert!(game.active_player == PLAYER_HOST && host_player.owner == player_addr, error::not_found(EPLAYER_NOT_YOUR_TURN));
        assert!(game.active_player == PLAYER_GUEST && guest_player.owner == player_addr, error::not_found(EPLAYER_NOT_YOUR_TURN));

        let roll = randomness::u8_range(0, 6);	        

        if(game.active_player == PLAYER_HOST){
            vector::push_back(&mut game.host_dices,roll);            
        }else{
            vector::push_back(&mut game.guest_dices,roll);
        }        
    }

    /*
     * @notice user who initiates game is automatically host_player
     */
    fun choose_host_player(game: &mut Game, user: address) {
        assert!(!game.is_game_over, error::invalid_argument(EGAME_HAS_ALREADY_FINISHED));
        assert!(option::is_none(&game.host_player), error::already_exists(EPLAYER_TAKEN));

        game.host_player = option::some(Player {
            type: PLAYER_HOST,
            owner: user,
        });
    }

    /*
     * @notice another user whose not the creator may join as guest_player
     */
    fun choose_guest_player(game: &mut Game, user: address) {
        assert!(!game.is_game_over, error::invalid_argument(EGAME_HAS_ALREADY_FINISHED));
        assert!(option::is_none(&game.guest_player), error::already_exists(EPLAYER_TAKEN));

        game.guest_player = option::some(Player {
            type: PLAYER_GUEST,
            owner: user,
        });
    }

    /*
     * @notice place (x,y) move on a 3x3 board
     */
    fun place_move(game: &mut Game, tower_index: u8, dice_index: u8) {
        // validate game state
        assert!(!game.is_game_over, error::invalid_argument(EGAME_HAS_ALREADY_FINISHED));

        // validate player move
        // assert!(player_type == game.active_player, error::unauthenticated(EOUT_OF_TURN_MOVE));

        
        let tower = vector::borrow_mut(&mut game.board.towers, tower_index as u64);

        // validate cell is empty
        // assert!(vector::length() >0 && game.board.towers[tower_index].top().unwrap() == game.active_player , error::invalid_state(EINVALID_MOVE));                                

        tower.nuts = tower.nuts - 1;
        let dices: &vector<u8>;
        if(game.active_player == PLAYER_HOST){
            dices = &game.host_dices;
        }else{
            dices = &game.guest_dices;
        };

        let dice_num = *vector::borrow(dices,dice_index as u64);
        {
            let source_tower = vector::borrow_mut(&mut game.board.towers,tower_index as u64);
            source_tower.nuts = source_tower.nuts - 1;
        };
        
        {
            let dest_tower = vector::borrow_mut(&mut game.board.towers,(tower_index + dice_num) as u64);  
            dest_tower.nuts = dest_tower.nuts + 1;
        };        
        
    }	

    fun can_bear_off(game: &mut Game): bool {
        let bar = &game.board.bar;
        let bar_nuts: u8;
        if(game.active_player == PLAYER_GUEST){
            bar_nuts = bar.guest_nuts;
        }else{
            bar_nuts = bar.host_nuts;
        };
        
        if (bar_nuts > 0){
            return false
        };

        let towers = & game.board.towers;
        
            
        for (player_tower_index in 0..18){        
            let global_tower_index = global_to_player_tower_index(game.active_player,player_tower_index as u8);
            let tower = vector::borrow(towers,global_tower_index as u64);
            if(tower.owner == game.active_player && tower.nuts>0){
                return false
            }            
        };
        true                    
    }
    
    fun dice_is_valid(game: &mut Game,dice_index:u8) : bool{        
        if (dice_index < 0){
            return false
        };            
        if(game.active_player == PLAYER_GUEST && !((dice_index as u64) < vector::length(&game.guest_dices))){
            return false
        };
        if(game.active_player == PLAYER_HOST && !((dice_index  as u64) < vector::length(&game.host_dices))){
            return false
        };                  
        true
    }

    fun tower_index_is_valid(tower_index: u8): bool{
        tower_index >= 0 && tower_index < TOWERS_COUNT
    }

    /*
	* player can bear off the 
	*/
	public entry fun bear_off(game_addr: address, player_tower_index: u8,dice_index:u8) acquires Game {
        let game = borrow_global_mut(game_addr); 
        assert!(can_bear_off(game), error::invalid_argument(EBEAR_OFF_ERROR_NUTS_OUT_OF_HOME));
        assert!(dice_is_valid(game,dice_index) ,error::invalid_argument(EINVALID_DICE_INDEX));
        assert!(tower_index_is_valid(player_tower_index) ,error::invalid_argument(EINVALID_TOWER_INDEX));
        
        let dices: &vector<u8>;
        if(game.active_player == PLAYER_HOST){
            dices = &game.host_dices;
        }else{
            dices = &game.guest_dices;
        };

        let dice_num = *vector::borrow(dices,dice_index as u64);

        assert!(dice_num + player_tower_index < TOWERS_COUNT ,error::invalid_argument(EINVALID_DICE_NUM));
        let global_tower_index = global_to_player_tower_index(game.active_player,player_tower_index);

        let tower = vector::borrow_mut(&mut game.board.towers,global_tower_index as u64);         
        
        assert!(tower.owner != game.active_player ,error::invalid_argument(EINVALID_TOWER_PLAYER));
        tower.nuts = tower.nuts - 1;
    }

    /*
     * @notice player who has no nut in the board, and on the bar , wins the game
     */
    fun check_player_win(game: &mut Game): bool {					
        if(game.active_player == PLAYER_HOST){
            if(game.board.bar.host_nuts > 0){
                return false
            };
        }else{
            if(game.board.bar.guest_nuts > 0){
                return false
            };
        };

        for (tower_index in 0..TOWERS_COUNT) {
            let tower = vector::borrow(&game.board.towers,tower_index as u64);
            if(tower.owner == game.active_player && tower.nuts>0){
                return false             
            }
                
		};
		true		
    }

    /*
     * @notice check status of game
     */
    fun check_is_game_over(game: &Game): bool {
        game.is_game_over
    }

    /*
     * @notice helper function to destroy Game at the end of session / forfeit
     */
    // fun cleanup_game(game: Game) {
        // let Game {
        //     board: Board {
        //         towers,
        //         bar
        //     },
        //     host_player,
        //     guest_player,
        //     active_player: _,
        //     is_game_over: _,
        // } = game;
        // option::destroy_some(host_player);
        // option::destroy_some(guest_player);        
    // }   
}