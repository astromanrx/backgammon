// Game rules reference: https://www.bkgm.com/rules.html

module backgammon::ttt {
    use aptos_framework::account;
    use aptos_framework::event;
    use std::error;
    use std::option::{Self, Option};
    use std::signer;
    use std::vector;
	use aptos_framework::randomness;

    //// GAME CONSTANTS    
    const TOWERS_COUNT: u8 = 24
    //// PLAYER CONSTANTS
    const PLAYER_X_TYPE: u8 = 0;
    const PLAYER_O_TYPE: u8 = 1;

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
        type: u64,
        owner: address,
    }

    //TODO: simplify the tower to (u8,u8) for player index and num of the player's nuts in the tower
    struct Board has drop, store {
        towers: [Vec<u8>;TOWERS_COUNT],  
		bar: [Vec<u8>;2],		
    }	

    //// to access game records after games are over.
    struct Game has key, store {
        board: Board,
		dices:[Vec<u8>,2]; 
        player_x: Option<Player>,
        player_o: Option<Player>,
        active_player: u8,
        is_game_over: bool,
    }

    /*
     * @notice initializes a valid, playable Game
     * @dev stores the Game into global storage
     */
    //// TODO: have Game as its own object, with its own address
    public entry fun start_game(creator: &signer) {
        // check game doesn't already exist under creator address
        assert!(!exists<Game>(signer::address_of(creator)), error::already_exists(EGAME_ALREADY_EXISTS_FOR_USER));
        let game = initalize_game(creator);
        let creator_addr = signer::address_of(creator);
        choose_player_x(&mut game, creator_addr);
        move_to<Game>(creator, game);        
    }

    /*
     * @notice lets another user join given a valid game address
     */
    public entry fun join_as_player_o(new_user: &signer, game_addr: address) acquires Game {
        let new_user_addr = signer::address_of(new_user);
        assert!(new_user_addr != game_addr, error::invalid_argument(ECANNOT_JOIN_AS_TWO_PLAYERS));

        assert!(exists<Game>(game_addr), error::not_found(EGAME_DOESNT_EXIST));
        let game = borrow_global_mut(game_addr);
        choose_player_o(game, new_user_addr);
    }

    /*
     * @notice places a move at a given (x,y) coordinate on a 3x3 board
     * @dev checks to ensure a player can make a valid move
     */
    public entry fun choose_move(player: &signer, game_addr: address, tower_index: u8, dice_index: u8) acquires Game {
        assert!(x < 3, error::out_of_range(EOUT_OF_BOUNDS_MOVE));
        assert!(y < 3, error::out_of_range(EOUT_OF_BOUNDS_MOVE));
        let game: &mut Game = borrow_global_mut(game_addr);
        let player_x = option::borrow_mut(&mut game.player_x);
        let player_o = option::borrow_mut(&mut game.player_o);

        let player_addr = signer::address_of(player);
        assert!(
            player_addr != player_x.owner || player_addr != player_o.owner,
            error::permission_denied(EPLAYER_NOT_IN_GAME),
        );

        if (player_addr == player_x.owner) {
            place_move(game, tower_index, dice_index, *player_x);
        } else {
            place_move(game, tower_index, dice_index, *player_o);
        };

        if(game.dices[game.active_player].len() == 0)
            event::emit(EndOfTurnEvent { game_address: game_addr, });
    }

    /*
     * @notice destroy Game at the end of session / forfeit
     */
    public entry fun cleanup(creator: &signer) acquires Game {
        let creator_addr: address = signer::address_of(creator);
        // abort if no such game exists under creator
        let game: Game = move_from<Game>(creator_addr);
        cleanup_game(game);
    }

    /*
     * @notice voluntarily give up, the other player wins
     */
    public entry fun forfeit(player: &signer, game_addr: address) acquires Game {
        let player_addr = signer::address_of(player);
        let game: &mut Game = borrow_global_mut(game_addr);
        let player_x = option::borrow_mut(&mut game.player_x);
        let player_o = option::borrow_mut(&mut game.player_o);

        assert!(
            player_addr != player_x.owner || player_addr != player_o.owner,
            error::permission_denied(EPLAYER_NOT_IN_GAME)
        );
		
		//TODO: must determine player that requested the forfeit and set it as the loser

        game.is_game_over = true;

        event::emit(GameOverEvent { game_address: game_addr, is_game_over: true, });
    }
	
	fun player_to_global_tower_index(player:u8, tower_index: u8){
		if(player == PLAYER_X_TYPE)
			tower_index
		else
			TOWERS_COUNT - tower_index - 1
		
	}

    fun global_to_player_tower_index(player:u8, tower_index: u8){
		if(player == PLAYER_X_TYPE)
			TOWERS_COUNT - tower_index - 1
		else
			tower_index
		
	}

	fun push_nut(game: &mut Game,player: u8,player_tower_index:u8 , count: u8 = 1){
		let global_tower_index = player_to_global_tower_index(player,tower_index)
        for i in 0..count:
		    game.board.towers[global_tower_index].push(player);		
	}
		

    /*
     * @notice initialize Game struct with base values for a 3x3 game
     */
    fun initalize_game(creator: &signer): Game {		
        let towers: [Vec<u8>;TOWERS_COUNT];
		let bar: [Vec<u8>;2];
		
		for i in 0..TOWERS_COUNT:
			towers[i] = Vec::empty<u64>()		
		
		bars = [Vec::empty<u64>(),Vec::empty<u64>()];
        
        let game = Game {
            board: Board {
                towers: towers,                
				bar: bar
            },
            player_x: option::none(),
            player_o: option::none(),
            active_player: PLAYER_X_TYPE,
            is_game_over: false,
        }
		
		for player in PLAYER_X_TYPE..PLAYER_O_TYPE{            
            push_nut(game,player,0,2)            
            push_nut(game,player,11,5)            
            push_nut(game,player,16,3)            
            push_nut(game,player,18,5)
        }
			
		game
    }
	
	#[randomness(max_gas=56789)]
    entry fun roll_the_dice(player: signer) {
        let roll = randomness::u64_range(0, 6);
		
		let player_addr = signer::address_of(player);
        let game: &mut Game = borrow_global_mut(game_addr);
        let player_x = option::borrow_mut(&mut game.player_x);
        let player_o = option::borrow_mut(&mut game.player_o);
    }

    /*
     * @notice user who initiates game is automatically player_x
     */
    fun choose_player_x(game: &mut Game, user: address) {
        assert!(!game.is_game_over, error::invalid_argument(EGAME_HAS_ALREADY_FINISHED));
        assert!(option::is_none(&game.player_x), error::already_exists(EPLAYER_TAKEN));

        game.player_x = option::some(Player {
            type: PLAYER_X_TYPE,
            owner: user,
        });
    }

    /*
     * @notice another user whose not the creator may join as player_o
     */
    fun choose_player_o(game: &mut Game, user: address) {
        assert!(!game.is_game_over, error::invalid_argument(EGAME_HAS_ALREADY_FINISHED));
        assert!(option::is_none(&game.player_o), error::already_exists(EPLAYER_TAKEN));

        game.player_o = option::some(Player {
            type: PLAYER_O_TYPE,
            owner: user,
        });
    }

    /*
     * @notice place (x,y) move on a 3x3 board
     */
    fun place_move(game: &mut Game, tower_index: u8, dice_index: u8, player: Player) {
        // validate game state
        assert!(!game.is_game_over, error::invalid_argument(EGAME_HAS_ALREADY_FINISHED));

        // validate player move
        assert!(player_type == game.active_player, error::unauthenticated(EOUT_OF_TURN_MOVE));

        
        let tower = vector::borrow_mut(&mut game.board.towers, tower_index);

        // validate cell is empty
        assert!(game.board.towers[tower_index].len()>0 && game.board.towers[tower_index].top().unwrap() == game.active_player , error::invalid_state(EINVALID_MOVE));                                

        game.board.towers[tower_index].pop()
        let dice_num = game.board.dices[game.active_player]
        game.board.towers[tower_index + dice_num].push(game.active_player)
    }	

    fun can_bear_off(game: &mut Game) {
        if (game.board.bar[game.active_player].len() > 0)
            return false
        for player_tower_index in 0..18{
            let global_tower_index = global_to_player_tower_index(player_tower_index)
            if(game.board.towers[global_tower_index].len() > 0 && game.board.towers[global_tower_index][0] == game.active_player)
        }
        true                    
    }
    
    fun dice_is_valid(game: &mut Game,dice_index:u8){        
        if (dice_index < 0)
            return false
        if (dice_index > game.dices[game.active_player].len() - 1)
            return false
        true
    }

    fun tower_index_is_valid(tower_index: u8){
        tower_index >= 0 && tower_index < TOWERS_COUNT
    }

    /*
	* player can bear off the 
	*/
	fun bear_off(game: &mut Game,player_tower_index: u8,dice_index:u8) {
        assert!(can_bear_off(game), error::invalid_argument(EBEAR_OFF_ERROR_NUTS_OUT_OF_HOME))
        assert!(dice_is_valid() ,error::invalid_argument(EINVALID_DICE_INDEX))        
        assert!(tower_index_is_valid() ,error::invalid_argument(EINVALID_TOWER_INDEX))
        let dice_num = game.dices[game.active_player][dice_index]    
        assert!(dice_num + player_tower_index < TOWERS_COUNT ,error::invalid_argument(EINVALID_DICE_NUM))
        let global_tower_index = global_to_player_tower_index(player_tower_index)
        assert!(game.board.towers[global_tower_index].len() > 0 && game.board.towers[global_tower_index].top().unwrap() == game.active_player ,error::invalid_argument(EINVALID_TOWER_PLAYER))
		game.board.towers[global_tower_index].pop()
	}

    /*
     * @notice player who has no nut in the board, and on the bar , wins the game
     */
    fun check_player_win(game: &mut Game): bool {					
        for tower_index in 0..game.board.len() {
			if (game.board.bar[game.active_player].len() == 0 && game.board.towers[tower_index].len()>0 && game.board.towers[tower_index][0] == game.active_player)
				return false
		}
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
    fun cleanup_game(game: Game) {
        let Game {
            board: Board {
                towers,
                bar
            },
            player_x,
            player_o,
            active_player: _,
            is_game_over: _,
        } = game
        option::destroy_some(player_x)
        option::destroy_some(player_o)        
    }   
}