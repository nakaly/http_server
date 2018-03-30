
use std::net::TcpListener;
use std::thread;
use std::io::{Read};
use std::io;
use std::io::{BufReader};
use std::fs::File;
use std::path::PathBuf;
use parser;
use data::{Response, Status};
use handler::FileHandler;

pub fn server_start() -> io::Result<()> {
    let lis = TcpListener::bind("127.0.0.1:8080")?;

    for stream in lis.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,

            Err(e) => {
                println!("An error occured while accepting a connection!");
                continue;
            }
        };

        let _ = thread::spawn(
            move || -> io::Result<()> {
                use parser::ParseResult::*;
                let mut buf = Vec::new();

                loop {
                    let mut b = [0; 1024];
                    let n = stream.read(&mut b)?;
                    if n == 0 {
                        return Ok(());
                    } 
                    buf.extend_from_slice(&b[0..n]);

                    match parser::parse(buf.as_slice()) {
                        Partial => continue,
                        Error => {
                            let res = Response::new(Status::BadRequest);
                            res.print_http(&mut stream);
                            return Ok(());
                        },
                        Complete(req) => {

                            let mut path = req.path;

                            println!("path:{:?}", req.path);
                            println!("method:{:?}", req.method.get_code());
                            println!("headers:{:?}", req.headers);
                            println!("body:{:?}", req.body);



                            let hendler = FileHandler::new("./")?;

                            match hendler.handle(req.path) {
                                Ok(file) => {
                                    let mut file = file;
                                    let mut res = Response::new(Status::Ok);
                                    let mut body = Vec::new();
                                    file.read_to_end(&mut body)?;
                                    res.body = Some(body);
                                    res.print_http(&mut stream);
                                }
                                Err(ioerror) => {
                                    println!("error after handle");
                                    use self::io::ErrorKind::*;
                                    match ioerror.kind() {
                                        NotFound => {
                                            println!("not found");
                                            let res = Response::new(Status::NotFound);
                                            res.print_http(&mut stream);
                                            return Ok(())
                                            },
                                        PermissionDenied => {
                                            println!("bad request");
                                            let res = Response::new(Status::BadRequest);
                                            res.print_http(&mut stream);
                                            return Ok(())
                                        },
                                        _ => {
                                            println!("internal server error");
                                            let res = Response::new(Status::InternalServerError);
                                            res.print_http(&mut stream);
                                            return Ok(())
                                        }
                                    }
                                }
                            }

                            while path.starts_with("/") {
                                path = &path[1..];
                            }

                            let path = PathBuf::new().join(path).canonicalize()?;
                            let base_dir = PathBuf::new().join("./").canonicalize()?;

                            if !path.starts_with(&base_dir) {
                                println!("BadRequest");
                                return Ok(());
                            }

                            let file = File::open(path).expect("file not found");
                            let mut file = BufReader::new(file);

                            io::copy(&mut file, &mut stream)?;
                            return Ok(());
                        }
                    }
                    
                }
            }
        );
    }
    Ok(())
}