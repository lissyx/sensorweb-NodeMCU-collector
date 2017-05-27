extern crate hyper;
use self::hyper::header::{ContentLength, Location};
use self::hyper::method::Method;
use self::hyper::server::{Server, Request, Response};
use self::hyper::status::StatusCode;
use self::hyper::uri::RequestUri;

use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

fn http_handler(req: Request, mut res: Response) {
    debug!("Received HTTP: {} {}", req.method, req.uri);

    match (req.method, req.uri) {
        (Method::Get, RequestUri::AbsolutePath(path)) => {
            let req_path = Path::new(path.as_str());
            if Path::new("/") == req_path {
                *res.status_mut() = StatusCode::PermanentRedirect;
                res.headers_mut().set(Location("/index.html".to_owned()));
                return;
            }

            let uri_path = if req_path.has_root() {
                let static_p = Path::new("static");
                match req_path.strip_prefix("/") {
                    Ok(clean) => static_p.join(clean),
                    Err(err)  => static_p.join(""),
                }
            } else {
                Path::new("static").join(path.to_string())
            };

            debug!("Trying to open path: {:?} ({})", uri_path.to_str(), path.to_string());
            match File::open(uri_path.clone()) {
                Ok(file) => {
                    let mut buf_reader = BufReader::new(file);
                    let mut contents = String::new();
                    match buf_reader.read_to_string(&mut contents) {
                        Ok(read_bytes) => {
                            *res.status_mut() = StatusCode::Ok;
                            res.headers_mut().set(ContentLength(contents.len() as u64));

                            let mut res = res.start().unwrap();
                            res.write_all(contents.as_bytes()).unwrap();
                        },
                        Err(err)       => {}
                    }
                },
                Err(err) => {
                    debug!("Error on resource {:?}: {}", uri_path.to_str(), err);
                    *res.status_mut() = StatusCode::NotFound;
                }
            }
        },
        _           => {
            *res.status_mut() = StatusCode::MethodNotAllowed;
        }
    }
}

pub fn th_http_listener(http_bind: String) {
    info!("Http thread started: {}", http_bind);
    Server::http(http_bind).unwrap().handle(http_handler).unwrap();
}
