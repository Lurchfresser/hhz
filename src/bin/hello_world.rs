use rouille::Response;


fn main() {
    println!("Starting server on 0.0.0.0:42069");
    rouille::start_server("0.0.0.0:42069", move |_request| {
        Response::text("Hallöchen Welt")
    });
}