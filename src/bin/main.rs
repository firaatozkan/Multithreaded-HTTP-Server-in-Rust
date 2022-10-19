use web_server::{WebServer, HttpRequestTypes::GET};

fn main() {
    let mut x = WebServer::new(3000);
    x.add_callback(GET, "/", |_c| {
        _c.send("Hello World");
    });
    x.run();
}
