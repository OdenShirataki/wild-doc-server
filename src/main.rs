use std::io::{Error, Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use wild_doc::{
    WildDoc
    ,IncludeLocal
    ,IncludeAdaptor
};

fn main() {
    let dir="./ss-test/";
    if std::path::Path::new(dir).exists(){
        std::fs::remove_dir_all(dir).unwrap();
        std::fs::create_dir_all(dir).unwrap();
    }else{
        std::fs::create_dir_all(dir).unwrap();
    }
    if let Ok(ss)=WildDoc::new(dir,IncludeLocal::new(dir)){
        let ss=Arc::new(Mutex::new(ss));
        let listener=TcpListener::bind("localhost:51818").expect("Error. failed to bind.");
        for streams in listener.incoming(){
            match streams {
                Err(e) => { eprintln!("error: {}", e)},
                Ok(stream) => {
                    let ss=Arc::clone(&ss);
                    thread::spawn(move || {
                        handler(stream,ss).unwrap_or_else(|error| eprintln!("{:?}", error));
                    });
                }
            }
        }
    }
    
}

fn handler<T>(mut stream: TcpStream,ss:Arc<Mutex<WildDoc<T>>>) -> Result<(), Error> where T:IncludeAdaptor{
    loop{
        let mut buffer=Vec::new();
        let nbytes={
            let mut tcp_reader=BufReader::new(&stream);
            tcp_reader.read_until(0,&mut buffer)?
        };
        if nbytes==0{
            break;
        }
        if let Ok(sml)=std::str::from_utf8(&buffer){
            let r=ss.clone().lock().unwrap().exec(sml);
            stream.write(r.as_bytes())?;
            stream.write(&[0])?;
        }else{
            stream.write(b"Error")?;
            stream.write(&[0])?;
        }
        stream.flush().unwrap();
    }
    Ok(())
}