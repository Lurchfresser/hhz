import {Chess} from 'chess.js'

const chess = new Chess(
    "4k3/8/8/8/8/8/8/R3K3 w Q - 0 1"
)


chess.move("e1c1");

// console.log(chess.moves());

console.log(chess.fen({forceEnpassantSquare: true}));
// returns:
// rnbqkbnr/pppppppp/8/8/7P/8/PPPPPPP1/RNBQKBNR b KQkq - 0 1
// expected
// rnbqkbnr/pppppppp/8/8/7P/8/PPPPPPP1/RNBQKBNR b KQkq h3 0 1