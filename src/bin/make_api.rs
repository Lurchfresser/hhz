use rouille::input::json_input;
use rouille::{Response, router, try_or_400};
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Deserialize, Serialize)]
struct GameRequest {
    pub fen: String,  // Changed from &'a str to String
}

fn main() {
    println!("Starting server on 0.0.0.0:42069");
    rouille::start_server("0.0.0.0:42069", move |request| {
        println!("received request on url: {}", request.url());
        router!(request,
            // first route
            (POST) (/new-game) => {
                let game_request: GameRequest = try_or_400!(
                    json_input(
                        request,
                    )
                );
                println!("game fen: {}", game_request.fen);
                Response::text("").with_status_code(200)
            },

           _ => Response::text("Not found").with_status_code(404)

        )
    });
}
