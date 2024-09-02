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


    #[event]
    struct GameOverEvent has drop, store {
        game_address: address,
        is_game_over: bool,
    }

    struct Player has copy, drop, store {
        type: u64,
        owner: address,
    }

    struct Board has drop, store {
        towers: [vector<u8>;24],  
		bar: [vector<u8>;2],		
    }	

    //// to access game records after games are over.
    struct Game has key, store {
        board: Board,
		dices:[vector<u8>,2]; 
        player_x: Option<Player>,
        player_o: Option<Player>,
        is_player_x_turn: bool,
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
            place_move(game, x, y, *player_x);
        } else {
            place_move(game, x, y, *player_o);
        };
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
			24 - tower_index - 1
		
	}

	fun push_nut(game: &mut Game,player: u8,player_tower_index:u8){
		let global_tower_index = player_to_global_tower_index(player,tower_index)
		game.board.towers[global_tower_index].push(player);		
	}
		

    /*
     * @notice initialize Game struct with base values for a 3x3 game
     */
    fun initalize_game(creator: &signer): Game {		
        let towers: [Vec<u8>;24];
		let bar: [Vec<u8>;2];
		
		for i in 0..24:
			towers[i] = Vec::empty<u64>()		
		
		bars = [Vec::empty<u64>(),Vec::empty<u64>()];
        
        let game = Game {
            board: Board {
                towers: towers,                
				bar: bar
            },
            player_x: option::none(),
            player_o: option::none(),
            is_player_x_turn: true,
            is_game_over: false,
        }
		
		for i in 0..2
			push_nut(game,PLAYER_X_TYPE,0)
		for i in 0..5
			push_nut(game,PLAYER_X_TYPE,11)
		for i in 0..3
			push_nut(game,PLAYER_X_TYPE,16)
		for i in 0..5
			push_nut(game,PLAYER_X_TYPE,18)
			
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
        let player_type = player.type;
        if (game.is_player_x_turn) {
            assert!(player_type == PLAYER_X_TYPE, error::unauthenticated(EOUT_OF_TURN_MOVE));
        } else {
            assert!(player_type == PLAYER_O_TYPE, error::unauthenticated(EOUT_OF_TURN_MOVE));
        };

        let position = WIDTH_AND_HEIGHT * x + y;
        let cell = vector::borrow_mut(&mut game.board.vec, position);

        // validate cell is empty
        assert!(*cell == EMPTY_CELL, error::invalid_state(EINVALID_MOVE));
        *cell = player_type;

        // update turn after placing move
        if (game.is_player_x_turn) {
            game.is_player_x_turn = false;
        } else {
            game.is_player_x_turn = true;
        };

        // check if game won
        let is_game_over = check_player_win(game);
        if (is_game_over) game.is_game_over = true;
    }
	
	/*
	* player can bear off the 
	*/
	fun bear_off(game: &mut Game,dice_index:u8) {
		for i in 0..game.board.len() {
			
		}
	}

    /*
     * @notice player who has no nut in the board, and on the bar , wins the game
     */
    fun check_player_win(game: &mut Game): bool {
		let active_player: u8;
		
		if (game.is_player_x_turn) {
            active_player = PLAYER_X_TYPE;
        } else {
            active_player = PLAYER_O_TYPE;
        };
			
        for tower_index in 0..game.board.len() {
			if (game.board.bar[active_player].len() == 0 && game.board.towers[tower_index].len()>0 && game.board.towers[tower_index][0] == active_player)
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
            is_player_x_turn: _,
            is_game_over: _,
        } = game;
        option::destroy_some(player_x);
        option::destroy_some(player_o);
        while (!vector::is_empty(&vec)) {
            vector::pop_back(&mut vec);
        };
    }   
}