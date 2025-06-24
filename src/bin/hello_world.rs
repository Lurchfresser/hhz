use rouille::Request;
use rouille::Response;


fn main() {
    rouille::start_server("localhost:42069", move |request| {

        Response::text("hello world")
    });
}