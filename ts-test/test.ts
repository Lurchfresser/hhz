import {Chess} from 'chess.js'

const chess = new Chess(
    "8/8/8/8/4k1pR/8/5P2/1K6 w - - 0 1"
)


chess.move("f2f4");

console.log(chess.fen({forceEnpassantSquare: true}));
// returns:
// rnbqkbnr/pppppppp/8/8/7P/8/PPPPPPP1/RNBQKBNR b KQkq - 0 1
// expected
// rnbqkbnr/pppppppp/8/8/7P/8/PPPPPPP1/RNBQKBNR b KQkq h3 0 1