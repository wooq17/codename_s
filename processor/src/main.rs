extern crate iron;
extern crate time;
extern crate router;
extern crate bodyparser;
extern crate persistent;
extern crate bincode;
extern crate rustc_serialize;

use iron::prelude::*;
use iron::status;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use router::{Router};
use persistent::Read;
use time::precise_time_ns;
use bincode::rustc_serialize::{encode, decode};

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

#[derive(Debug, Clone, RustcDecodable)]
struct profile {
    name: String,
    age: String,
    nickname: Option<String>,
}

fn log_body(req: &mut Request) -> IronResult<Response> {
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

    let profile_body = req.get::<bodyparser::Struct<profile>>();
    let mut response_string = String::from("fail");
    match profile_body {
        Ok(Some(profile_body)) => {
            println!("Parsed body:\n{:?}", profile_body);
            response_string = profile_body.name + " is " + &profile_body.age  + " years old";
        },
        Ok(None) => println!("No body"),
        Err(err) => println!("Error: {:?}", err)
    }

    Ok(Response::with((status::Ok, response_string)))
}

const MAX_BODY_LENGTH: usize = 1024 * 1024 * 10;

// basic GET request
fn get_handler(req: &mut Request) -> IronResult<Response> {
    println!("[DEBUG] basic GET request");
    Ok(Response::with((status::Ok, "Hello world!")))
}

// GET request with query string
// failed...
fn get_with_query_handler(req: &mut Request) -> IronResult<Response> {
    println!("[DEBUG] GET request with query string");
    let ref query = req.extensions.get::<Router>().unwrap().find("query").unwrap_or("fail to unwrap");
    Ok(Response::with((status::Ok, *query)))
}

// basic POST request
fn pst_handler(req: &mut Request) -> IronResult<Response> {
    println!("[DEBUG] basic GET request");
    Ok(Response::with((status::Ok, "Hello world!")))
}

fn main() {
    // create router
    let mut router = Router::new();
    router.get("/get", get_handler);
    // router.get("/get_with_query:query", get_with_query_handler);
    router.post("/post", log_body);

    // add chain activities
    let mut chain = Chain::new(router);
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);
    
    // run the server
    Iron::new(chain).http("localhost:3000").unwrap();
    println!("server started....");
}