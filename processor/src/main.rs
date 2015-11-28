extern crate iron;
extern crate time;
extern crate router;
extern crate bodyparser;
extern crate persistent;
extern crate interface;

use iron::prelude::*;
use iron::status;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use router::{Router};
use time::precise_time_ns;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use interface::*;




pub struct Writer {
    pub tx: Sender<Log>,
    pub rx: Receiver<Log>
}

impl Writer {
    pub fn new() -> Writer {
        let (_tx, _rx): (Sender<Log>, Receiver<Log>) = mpsc::channel();
        Writer{ tx: _tx, rx: _rx }
    }
    
    pub fn get_transmitter(&self) -> Option<Sender<Log>> {
        Some(self.tx.clone())
    }

	pub fn write() {
        let path = Path::new("out/lorem_ipsum.txt");
        let display = path.display();
    
        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}",
                            display,
                            Error::description(&why)),
            Ok(file) => file,
        };
    
        // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
        match file.write_all("sample string".as_bytes()) {
            Err(why) => {
                panic!("couldn't write to {}: {}", display,
                                                Error::description(&why))
            },
            Ok(_) => println!("successfully wrote to {}", display),
        }
    }
}












struct ResponseTime;

impl typemap::Key for ResponseTime { type Value = u64; }

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Ok(res)
    }
}

/// upload log data
fn upload_handler(req: &mut Request) -> IronResult<Response> {
    let body = req.get::<bodyparser::Raw>();
    match body {
        Ok(Some(body)) => println!("Read body:\n{}", body),
        Ok(None) => println!("No body"),
        Err(err) => println!("Error: {:?}", err)
    }

    let json_body = req.get::<bodyparser::Json>();
    match json_body {
        Ok(Some(json_body)) => println!("Parsed body:\n{}", json_body),
        Ok(None) => println!("No body"),
        Err(err) => println!("Error: {:?}", err)
    }

    let parsed_log = req.get::<bodyparser::Struct<interface::Log>>();
    let mut response_string = String::from("fail");
    match parsed_log {
        Ok(Some(log)) => {
            println!("Parsed body:\n{:?}", log);
            // process the log data...
            
            response_string = String::from("SUCCESS");
        },
        Ok(None) => println!("No body"),
        Err(err) => println!("Error: {:?}", err)
    }

    Ok(Response::with((status::Ok, response_string)))
}

// basic GET request
fn check_handler(req: &mut Request) -> IronResult<Response> {
    println!("[DEBUG] basic GET request");
    Ok(Response::with((status::Ok, "Hello world!")))
}

fn main() {
    // create router
    let mut router = Router::new();
    router.get("/get", check_handler); // for debug
    router.put("/log/upload", upload_handler);

    // add chain activities
    let mut chain = Chain::new(router);
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);
    
    // run the server
    Iron::new(chain).http("localhost:3000").unwrap();
    println!("server started....");
}