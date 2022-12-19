use std::collections::HashMap;
use std::io::{BufRead, BufReader, Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[macro_use]
extern crate serde_derive;

use wild_doc::{IncludeAdaptor, WildDoc};

mod include;
use include::{IncludeEmpty, IncludeRemote};

#[derive(Deserialize)]
struct Config {
    wilddoc: Option<ConfigWildDoc>,
}
#[derive(Deserialize)]
struct ConfigWildDoc {
    path: Option<String>,
    bind_addr: Option<String>,
    port: Option<String>,
    delete_dir_on_start:Option<String>
}

fn main() {
    if let Ok(mut f) = std::fs::File::open("wild-doc.toml") {
        let mut toml = String::new();
        if let Ok(_) = f.read_to_string(&mut toml) {
            let config: Result<Config, toml::de::Error> = toml::from_str(&toml);
            if let Ok(config) = config {
                if let Some(config) = config.wilddoc {
                    if let (Some(dir), Some(bind_addr), Some(port)) =
                        (config.path, config.bind_addr, config.port)
                    {
                        if let Some(delete_dir_on_start)=config.delete_dir_on_start{
                            if delete_dir_on_start=="1"{
                                if std::path::Path::new(&dir).exists() {
                                    std::fs::remove_dir_all(&dir).unwrap();
                                }
                            }
                        }

                        let mut wild_docs = HashMap::new();
                        let listener = TcpListener::bind(&(bind_addr + ":" + &port))
                            .expect("Error. failed to bind.");
                        for streams in listener.incoming() {
                            match streams {
                                Err(e) => {
                                    eprintln!("error: {}", e)
                                }
                                Ok(stream) => {
                                    let mut dbname = Vec::new();
                                    let mut tcp_reader = BufReader::new(&stream);
                                    let nbytes = tcp_reader.read_until(0, &mut dbname).unwrap();
                                    if nbytes > 0 {
                                        dbname.remove(dbname.len() - 1);
                                        if let Ok(dbname) = std::str::from_utf8(&dbname) {
                                            let dir = dir.to_owned() + dbname + "/";
                                            let wd =
                                                wild_docs.entry(dir).or_insert_with_key(|dir| {
                                                    if !std::path::Path::new(dir).exists() {
                                                        std::fs::create_dir_all(dir).unwrap();
                                                    }
                                                    Arc::new(Mutex::new(
                                                        WildDoc::new(dir, IncludeEmpty::new())
                                                            .unwrap(),
                                                    ))
                                                });
                                            let wd = Arc::clone(&wd);
                                            thread::spawn(move || {
                                                handler(stream, wd).unwrap_or_else(|error| {
                                                    eprintln!("handler {:?}", error)
                                                });
                                            });
                                        }
                                    } else {
                                        println!("recv 0 bytes");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn handler<T: IncludeAdaptor>(
    mut stream: TcpStream,
    wd: Arc<Mutex<WildDoc<T>>>,
) -> Result<(), Error> {
    stream.write_all(&[0])?;

    let mut writer = stream.try_clone().unwrap();
    let mut tcp_reader = BufReader::new(&stream);
    loop {
        let mut input_json = Vec::new();
        let nbytes = tcp_reader.read_until(0, &mut input_json)?;
        if nbytes == 0 {
            break;
        }
        input_json.remove(input_json.len() - 1);

        let mut xml = Vec::new();
        let nbytes = tcp_reader.read_until(0, &mut xml)?;
        if nbytes == 0 {
            break;
        }
        xml.remove(xml.len() - 1);

        if let (Ok(input_json), Ok(xml)) =
            (std::str::from_utf8(&input_json), std::str::from_utf8(&xml))
        {
            let mut include = IncludeRemote::new(stream.try_clone().unwrap());
            let r = wd.clone().lock().unwrap().run_specify_include_adaptor(
                xml,
                input_json,
                &mut include,
            )?;
            writer.write(&[0])?;
            writer.write(r.body())?;
            writer.write(&[0])?;
            writer.write(r.options_json().as_bytes())?;
            writer.write(&[0])?;
        } else {
            writer.write(b"Error")?;
            writer.write(&[0])?;
        }
    }
    Ok(())
}
