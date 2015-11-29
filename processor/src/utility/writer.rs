extern crate interface;

use std::io::prelude::*;
use std::fs::File;
use interface::*;

pub struct Writer {
	use std::sync::mpsc;
	use std::sync::mpsc::{Sender, Receiver};
    pub tx: Sender<Log>,
    pub rx: Receiver<Log>
}

impl Writer {
	
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
    match file.write_all(LOREM_IPSUM.as_bytes()) {
        Err(why) => {
            panic!("couldn't write to {}: {}", display,
                                               Error::description(&why))
        },
        Ok(_) => println!("successfully wrote to {}", display),
    }
}