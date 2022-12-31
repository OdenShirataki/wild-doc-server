use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    path::{Path, PathBuf},
};

use wild_doc::IncludeAdaptor;

pub struct IncludeEmpty {}
impl IncludeEmpty {
    pub fn new() -> Self {
        Self {}
    }
}
impl IncludeAdaptor for IncludeEmpty {
    fn include<P: AsRef<Path>>(&mut self, _: P) -> &str {
        ""
    }
}

pub struct IncludeRemote {
    stream: TcpStream,
    cache: HashMap<PathBuf, String>,
}
impl IncludeRemote {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            cache: HashMap::default(),
        }
    }
}
impl<'a> IncludeAdaptor for IncludeRemote {
    fn include<P: AsRef<Path>>(&mut self, path: P) -> &str {
        let path = path.as_ref().to_path_buf();
        self.cache
            .entry(path.to_owned())
            .or_insert_with_key(|path| {
                self.stream.write(("include:".to_owned() + path.to_str().unwrap()).as_bytes()).unwrap();
                self.stream.write(&[0]).unwrap();
                let mut reader = BufReader::new(&self.stream);
                let mut recv_response = Vec::new();
                if let Ok(v) = reader.read_until(0, &mut recv_response) {
                    if v > 0 {
                        recv_response.remove(recv_response.len() - 1);
                        if let Ok(xml) = std::string::String::from_utf8(recv_response) {
                            return xml;
                        }
                    }
                }
                "".to_string()
            })
    }
}
