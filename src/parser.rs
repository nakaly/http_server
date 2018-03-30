

use std::str::from_utf8;
use std::collections::HashMap;
use data::{Request, Method};

pub enum ParseResult<T> {
    Complete(T),
    Partial,
    Error,
}

impl<T> ParseResult<T> {
    fn is_complete(&self) -> bool {
        use self::ParseResult::*;
        match *self {
            Complete(_) => true,
            _ => false,
        }
    }

    fn is_partial(&self) -> bool {
        use self::ParseResult::*;
        match *self {
            Partial => true,
            _ => false,
        }
    }
}

macro_rules! ptry {
    ( $x:expr ) => {
         match $x {
            ParseResult::Complete(v) => v,
            _ => return ParseResult::Error,
        }
    }
}

impl<T, E> From<Result<T,E>> for ParseResult<T> {
    fn from(r: Result<T, E>) -> Self {
        use self::ParseResult::*;
        match r {
            Ok(t) => Complete(t),
            Err(_) => Error,
        }
    }
}



pub fn parse(buf: &[u8]) -> ParseResult<Request> {
    use self::ParseResult::*;
    match parse_10(buf) {
        Complete(t) => Complete(t),
        Partial => Partial,
        Error => parse_09(buf),

    }
}

fn parse_09(mut buf: &[u8]) -> ParseResult<Request> {
    use self::ParseResult::*;

    let get = b"GET ";
    let end = b"\r\n";

    if !buf.starts_with(get) {
        return Error;
    }

    buf = &buf[get.len()..];
    if buf.ends_with(end) {
        buf = &buf[0..buf.len() - end.len()]
    } else {
        return Partial;
    }

    from_utf8(buf)
    .map(|value| Request::new(Method::Get, value)) 
    .into()
}



fn parse_10(mut buf: &[u8]) -> ParseResult<Request> {


    let buf = & mut buf;
    let method = ptry!(parse_method(buf));

    let () = ptry!(skip_space( buf));
    let path = ptry!( parse_path( buf));
    let mut request = Request::new( method, path);
    let () = ptry!(skip_space( buf));
    let () = ptry!(parse_http10_version(buf));
    let () = ptry!(parse_crlf(buf));
    let mut headers = HashMap::new();
    let mut content_length: Option<usize> = None;
    while(!buf.starts_with(b"\r\n")) {
        let (name, value) = ptry!(parse_header(buf));
        if name == "Content-Length" {
            match value {
                None => return ParseResult::Error,
                Some(v) => {
                    let v : &str = ptry!( from_utf8(v).into());
                    content_length = Some(ptry!(v.parse().into()));
                }
            }
        }

        let prev = headers.insert(name, value);
        if let Some(prev) = prev {
            println!(
                "[WARN] duplicated header: {}. discarding previous value: {:?}",
                name,
                prev
            )
        }

    }
    request.headers = headers;
    let () = ptry!(parse_crlf(buf));
    if let Some(size) = content_length {
        let body = ptry!(parse_body(buf, size));
        request.body = Some(body);
    }

    ParseResult::Complete(request)
}

fn parse_method<'a>(mut buf: & mut &'a [u8]) -> ParseResult<Method<'a>>  {
    use self::Method::*;
    
    let pos = match buf.iter().position(|&c| c == ' ' as u8 ) {
        Some(p) => p,
        None => return ParseResult::Error,
    };

    let method = match &buf[0..pos] {
        b"GET" => Get,
        b"HEAD" => Head,
        b"POST" => Post,
        other => {
            let other_str = ptry!(from_utf8(other).into());
            Ext(other_str)
        },
    };

    *buf = &buf[pos..];

    ParseResult::Complete(method)
}

fn parse_path<'a>(mut buf: &mut &'a [u8]) -> ParseResult<&'a str> {
    let pos = match buf.iter().position(|c| b" \t\r\n".contains(c) ) {
        Some(p) => p,
        None => return ParseResult::Error,
    };

    let path = &buf[0..pos];

    *buf = &buf[pos..];
    let result =  ptry!(from_utf8(path).into());
    ParseResult::Complete(result)
}
fn skip_space<'a>(mut buf: &mut &'a [u8]) -> ParseResult<()> {
    parse_fixed(buf, b" ")
}
fn parse_http10_version<'a>(mut buf: &mut &'a [u8]) -> ParseResult<()> {
    parse_fixed(buf, b"HTTP/1.0")
}
fn parse_crlf<'a>(mut buf: &mut &'a [u8]) -> ParseResult<()> {
    parse_fixed(buf, b"\r\n")
}
fn parse_colon<'a>(mut buf: &mut &'a [u8]) -> ParseResult<()> {
    parse_fixed(buf, b": ")
}
fn parse_fixed<'a>(mut buf: &mut &'a [u8], fixed: &[u8]) -> ParseResult<()> {
    if buf.starts_with(fixed) {
        *buf = &buf[fixed.len()..];
        return ParseResult::Complete(())
    } else {
        return ParseResult::Error
    }
}
fn parse_token<'a>(mut buf: &mut &'a [u8]) -> ParseResult<&'a str> {
    let token_chars = br#"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890!#$%&'*+-.^_`|~"#;
    let mut pos = 0;
    while pos < buf.len() && token_chars.contains(&buf[pos]) {
        pos += 1;
    }
    let token = &buf[0..pos];
    *buf = &buf[pos..];
    ParseResult::Complete(from_utf8(token).unwrap())
}
fn parse_header<'a>(mut buf: &mut &'a [u8]) -> ParseResult<(&'a str, Option<&'a [u8]>)> {
    let name = ptry!(parse_token(buf));
    let () = ptry!(parse_colon(buf));
    let mut value = None;
    if buf.starts_with(b"\r\n") {
        return ParseResult::Complete((name, value));
    }
    
    let pos = match buf.iter().position(|c| b"\r".contains(c) ) {
        Some(p) => p,
        None => return ParseResult::Error,
    };
    if buf[pos + 1] == b'\n' {
        value = Some(&buf[0..pos]);
        *buf = &buf[pos..];
    } else {
        return ParseResult::Error;
    }
    let () = ptry!(parse_crlf(buf));
    ParseResult::Complete((name, value))
}

fn parse_body<'a>(mut buf: &mut &'a [u8], size: usize) -> ParseResult<&'a [u8]> {
    if size <= buf.len() {
        let body = &buf[0..size];
        *buf = &buf[size..];
        ParseResult::Complete(body)
    } else {
        ParseResult::Partial
    }
}

#[test]
fn test_parse_header() {
    let header = b"Header: Authorization: Bearer token\r\n";
    let header = &mut &header[..];  
    let (name, value) = match parse_header(header) {
        ParseResult::Complete((n, v)) => (n, v),
        _ => return assert!(false),
    };
    assert_eq!(name, "Header");
    assert_eq!(value.unwrap(), b" Authorization: Bearer token");
}

#[test]
fn http10_parse_method() {
    let request =  b"GET /foo/bar HTTP/1.0\r\n";
    let request = & mut & request[..];
    let method = parse_method(request);
    let result = match method {
        ParseResult::Complete(Method::Get) => {
            true
            },
        _ => {
            println!("not matched");
            false
            },
    };
    assert!(result);
    let result = match skip_space(request) {
        ParseResult::Complete(()) => true,
        _ => false,
    };
    assert!(result);
    let path = match parse_path(request) {
        ParseResult::Complete(p) => p,
        _ => "",
    };
    assert_eq!(path, "/foo/bar");
    let result = match skip_space(request) {
        ParseResult::Complete(()) => true,
        _ => false,
    };
    assert!(result);
    let result = match parse_http10_version(request){
        ParseResult::Complete(()) => true,
        _ => false,
    };
    assert!(result);
        let result = match parse_crlf(request){
        ParseResult::Complete(()) => true,
        _ => false,
    };
    assert!(result);
    

}

#[test]
fn http09_get_success_root() {
    let req = b"GET /\r\n";
    let res = parse(req);
    assert!(res.is_complete());
}

#[test]
fn http09_get_success_foo_bar() {
    let req = b"GET /foo/bar\r\n";
    let res = parse(req);
    assert!(res.is_complete());
}

#[test]
fn http09_get_partial_root() {
    let req = b"GET /\r";
    let res = parse(req);
    assert!(res.is_partial());
}

#[test]
#[should_panic(expected = "panic")]
fn http09_past_failure() {
    let req = b"POST /\r\n";
    let res = parse(req);
    assert!(res.is_complete(), "panic");
}