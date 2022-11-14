use std::io::{Error, Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use wild_doc::{
    WildDoc
    ,IncludeAdaptor
};

pub struct IncludeEmpty{}
impl IncludeEmpty{
    pub fn new()->Self{
        Self{}
    }
}
impl IncludeAdaptor for IncludeEmpty{
    fn include(&mut self,_:&str)->String{
        "".to_string()
    }
}

struct IncludeRemote{
    stream:TcpStream
}
impl IncludeRemote{
    pub fn new(stream:TcpStream)->Self{
        Self{
            stream
        }
    }
}
impl<'a> IncludeAdaptor for IncludeRemote{
    fn include(&mut self,path:&str)->String {
        let _=self.stream.write(("include:".to_owned()+path).as_bytes());
        let _=self.stream.write(&[0]);
        let mut reader = BufReader::new(&self.stream);
        let mut recv_response = Vec::new();
        if let Ok(v)=reader.read_until(0,&mut recv_response) {
            if v > 0 {
                recv_response.remove(recv_response.len()-1);
                if let Ok(xml)=std::string::String::from_utf8(recv_response){
                    return xml;
                }
            }
        }
        "".to_string()
    }
}

fn main() {
    let dir="./wd-test/";
    if std::path::Path::new(dir).exists(){
        std::fs::remove_dir_all(dir).unwrap();
        std::fs::create_dir_all(dir).unwrap();
    }else{
        std::fs::create_dir_all(dir).unwrap();
    }
    if let Ok(wd)=WildDoc::new(dir,IncludeEmpty::new()){
        let wd=Arc::new(Mutex::new(wd));
        let listener=TcpListener::bind("localhost:51818").expect("Error. failed to bind.");
        for streams in listener.incoming(){
            match streams {
                Err(e) => { eprintln!("error: {}", e)},
                Ok(stream)=>{
                    let wd=Arc::clone(&wd);
                    thread::spawn(move || {
                        handler(stream,wd).unwrap_or_else(|error| eprintln!("handler {:?}", error));
                    });
                }
            }
        }
    }
}

fn handler<T:IncludeAdaptor>(
    mut stream: TcpStream
    ,wd:Arc<Mutex<WildDoc<T>>>
)->Result<(), Error>{
    loop{
        let mut buffer=Vec::new();
        let nbytes={
            let mut tcp_reader=BufReader::new(&stream);
            tcp_reader.read_until(0,&mut buffer)?
        };
        if nbytes==0{
            break;
        }
        buffer.remove(buffer.len()-1);
        if let Ok(xml)=std::str::from_utf8(&buffer){
            let mut include=IncludeRemote::new(stream.try_clone().unwrap());
            let r=wd.clone().lock().unwrap().exec_specify_include_adaptor(xml,&mut include)?;
            stream.write(&[0])?;
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