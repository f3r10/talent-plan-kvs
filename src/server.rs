use std::net::TcpListener;

struct KvsServer {
    engine: String
}

impl KvsServer{
    fn new(engine: String) -> Self {
        KvsServer{engine}
    }

    fn run(addr: String) -> Result<(), String> {
        let listener = TcpListener::bind(addr);
        Ok(())
       
    }

}
