use std::net::TcpListener;

pub struct KvsServer {
    engine: String
}

impl KvsServer{
    pub fn new(engine: String) -> Self {
        KvsServer{engine}
    }

    pub fn run(self, addr: String) -> Result<(), String> {
        let listener = TcpListener::bind(addr).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            println!("Connection established")
        }
        Ok(())
    }

}
