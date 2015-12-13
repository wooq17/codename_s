extern crate iron;
extern crate time;
extern crate router;
extern crate bodyparser;
extern crate persistent;
extern crate interface;
extern crate rustc_serialize;

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
use std::fs::OpenOptions;
use std::path::Path;
use std::thread;
use interface::*;
use rustc_serialize::json;

pub struct Writer {
    pub file: File,
    pub tx: Sender<Log>,
    pub rx: Receiver<Log>
}

impl Writer {
    pub fn new() -> Writer {
        let path = Path::new("./out/log");
        let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(&path)
                        .unwrap();
        
        let (_tx, _rx): (Sender<Log>, Receiver<Log>) = mpsc::channel();
        Writer{ file: file, tx: _tx, rx: _rx }
    }
    
    pub fn get_transmitter(&self) -> Option<Sender<Log>> {
        Some(self.tx.clone())
    }

	pub fn write(&mut self, log: Log) {
        let encoded = json::encode(&log).unwrap();
        
        if let Err(e) = writeln!(self.file, "{}", &encoded) {
            println!("{}", e);
        }
    }
    
    pub fn cycle(&mut self) {
        loop {
            self.write(self.rx.recv().unwrap());
        }
    }
}

pub fn write_logs(mut writer: Writer) {
    writer.cycle();
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

static mut TX: Option<Sender<Log>> = None;

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
            // ....
            
            unsafe {
                match TX {
                    Some(transmitter) => { 
                        let _tx = transmitter.clone();
                        _tx.send(log); 
                    },
                    _ => { println!("Cannot get the transmitter"); }
                }
            }
            
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
    let mut writer = Writer::new();
    unsafe { TX = Some(writer.get_transmitter().unwrap()); }
    thread::spawn(move|| {
        write_logs(writer);
    });
    
    // create router
    let mut router = Router::new();
    router.get("/get", check_handler); // for debug
    router.put("/logs/upload", upload_handler);

    // add chain activities
    let mut chain = Chain::new(router);
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);
    
    // run the server
    Iron::new(chain).http("localhost:3000").unwrap();
    println!("server started....");
}