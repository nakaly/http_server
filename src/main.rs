mod parser;
mod data;
mod server;
mod handler;

fn main() {

    match server::server_start() {
        Ok(_) => (),
        Err(e) => println!("{:?}", e),
    }
}
