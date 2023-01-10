use std::{
    collections::HashMap,
    io::{BufReader, Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
};

use wild_doc::IncludeAdaptor;

pub struct IncludeEmpty {
    cache: Option<String>,
}
impl IncludeEmpty {
    pub fn new() -> Self {
        Self { cache: None }
    }
}
impl IncludeAdaptor for IncludeEmpty {
    fn include<P: AsRef<Path>>(&mut self, _: P) -> &Option<String> {
        &mut self.cache
    }
}

pub struct IncludeRemote {
    stream: TcpStream,
    cache: HashMap<PathBuf, Option<String>>,
}
impl IncludeRemote {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            cache: HashMap::default(),
        }
    }
}
impl IncludeAdaptor for IncludeRemote {
    fn include<P: AsRef<Path>>(&mut self, path: P) -> &Option<String> {
        let path = path.as_ref().to_path_buf();
        self.cache
            .entry(path.to_owned())
            .or_insert_with_key(|path| {
                if let Some(path_str) = path.to_str(){
                    if path_str.len() > 0 {
                        self.stream
                            .write(("include:".to_owned() + path_str).as_bytes())
                            .unwrap();
                        self.stream.write(&[0]).unwrap();
                        let mut reader = BufReader::new(&self.stream);

                        let mut exists: [u8; 1] = [0];
                        if let Ok(()) = reader.read_exact(&mut exists) {
                            let exists = u8::from_be_bytes(exists);
                            if exists == 1 {
                                let mut len: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
                                if let Ok(()) = reader.read_exact(&mut len) {
                                    let len = u64::from_be_bytes(len) as usize;
                                    let mut recv_response = Vec::<u8>::with_capacity(len);
                                    unsafe {
                                        recv_response.set_len(len);
                                    }
                                    if let Ok(()) = reader.read_exact(recv_response.as_mut_slice()) {
                                        if let Ok(xml) = std::string::String::from_utf8(recv_response) {
                                            return Some(xml);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                None
            })
    }
}
