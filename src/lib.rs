use std::{
    borrow::Cow,
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread, fs::{File}
};

use crossbeam::channel::unbounded;

#[derive(Clone, Copy)]
pub enum HttpRequestTypes {
    NONE = -1,
    GET,
    POST,
    NumOfRequestTypes,
}

pub struct Client {
    client: TcpStream,
    request_method: HttpRequestTypes,
    request_index: String,
}

impl Client {
    pub fn serve_file(&mut self, file_path: &str) {
        let mut file_handler = File::open(file_path).expect("File not found!");
        let mut file_content = String::new();
        file_handler.read_to_string(&mut file_content).expect("Error reading file!");
        let result = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", file_content.len(), file_content);
        self.client.write(result.as_bytes()).unwrap();
    }

    pub fn send(&mut self, string_: &str) {
        self.client.write(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", string_.len(), string_).as_bytes()).unwrap();
    }
}

pub struct WebServer {
    listener: TcpListener,
    callback_table: [HashMap<String, fn(&mut Client) -> ()>; 
                     HttpRequestTypes::NumOfRequestTypes as usize],
}

impl WebServer {
    pub fn new(port: i32) -> Self {
        WebServer {
            listener: TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap(),
            callback_table: [HashMap::new(), HashMap::new()],
        }
    }

    pub fn run(&self) {
        let (tx, rx) = unbounded::<Client>();

        for _i in 0..6 {
            let rx_clone = rx.clone();
            let callback_table_clone = self.callback_table.clone();

            thread::spawn(|| {
                WebServer::handle_client(rx_clone, callback_table_clone);
            });
        }

        loop {
            for i in self.listener.incoming() {
                match i {
                    Ok(new_client) => tx
                        .send(Client {
                            client: new_client,
                            request_index: "".to_string(),
                            request_method: HttpRequestTypes::NONE,
                        })
                        .unwrap(),
                    Err(_) => (),
                }
            }
        }
    }

    pub fn add_callback(&mut self, method: HttpRequestTypes, index: &str, callback: fn(&mut Client) -> ()) {
        self.callback_table[method as usize].insert(index.to_string(), callback);
    }

    fn handle_client(rx_clone: crossbeam::channel::Receiver<Client>,
                     callback_table_clone: [HashMap<String, fn(&mut Client)>; HttpRequestTypes::NumOfRequestTypes as usize]) {
        loop {
            let mut new_client = rx_clone.recv().unwrap();
            {
                let mut buffer = [0; 1024];
                new_client.client.read(&mut buffer).unwrap();
                WebServer::parse_request(&mut new_client, String::from_utf8_lossy(&buffer));
            }
            WebServer::operate_callbacks(&mut new_client, &callback_table_clone);
        }
    }

    fn parse_request(new_client: &mut Client, parsed_buffer: Cow<str>) -> () {
        let parsed_req = parsed_buffer.split(" ").collect::<Vec<&str>>();

        if parsed_req[0] == "GET" {
            new_client.request_method = HttpRequestTypes::GET;
        } else if parsed_req[0] == "POST" {
            new_client.request_method = HttpRequestTypes::POST;
        }

        new_client.request_index = parsed_req[1].to_string();
    }

    fn operate_callbacks(new_client: &mut Client, callback_table_clone: &[HashMap<String, fn(&mut Client)>; HttpRequestTypes::NumOfRequestTypes as usize]) {
        match callback_table_clone
            .get(new_client.request_method as usize)
            .unwrap()
            .get(&new_client.request_index)
        {
            Some(&callback) => callback(new_client),
            None => (),
        }
    }
}
