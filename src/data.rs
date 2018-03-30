use std::collections::HashMap;
use std::io::{Write, Result};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Method<'a> {
    Get,
    Head,
    Post,
    Ext(&'a str),
}

impl<'a> Default for Method<'a> {
    fn default() -> Self {
        Method::Get
    }
}

impl<'a> Method<'a> {
    pub fn  get_code(&'a self) -> &'a str {
        use self::Method::*;
        match *self {
            Get => "GET",
            Head => "HEAD",
            Post => "POST",
            Ext(m) => m,

        }
    }
} 

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Request<'a>{
    pub path: &'a str,
    pub method: Method<'a>,
    pub headers: HashMap<&'a str, Option<&'a [u8]>>,
    pub body: Option<&'a [u8]>,
}

impl<'a> Request<'a> {
    pub fn new(method: Method<'a>, path: &'a str) -> Self {
        Self {
            path,
            method,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Status {
    Ok,
    BadRequest,
    NotFound,
    InternalServerError,
}

impl<'a> Default for Status {
    fn default() -> Self {
        Status::Ok
    }
} 

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Version {
    HTTP09,
    HTTP10,
}
impl<'a> Default for Version {
    fn default() -> Self {
        Version::HTTP10
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Response {
    pub status: Status,
    pub version: Version,
    pub headers: HashMap<String, Option<Vec<u8>>>,
    pub body :  Option<Vec<u8>>,
}

impl Response {
    pub fn new(status: Status) -> Self {
        Self {
            status,
            ..Self::default()
        
        }
    }   
    pub fn print_http<W: Write>(&self, mut w: &mut W) -> Result<()>{
        println!("start print_http version:{:?}", self.version);
        match self.version {
            Version::HTTP09 => self.print_http09(w),
            Version::HTTP10 => self.print_http10(w),
        }
    }
    pub fn print_http09<W: Write>(&self, mut w: &mut W) -> Result<()>{
        println!("start print http09");
        if let Some(ref body) = self.body {
            w.write(body.as_ref())?;
        }
        Ok(())
    }
    pub fn print_http10<W: Write>(&self, mut w: &mut W) -> Result<()>{
        println!("start print http10");
        match self.status {
            Status::Ok => write!(w, "HTTP/1.0 200 Ok\r\n"),
            Status::BadRequest => write!(w, "HTTP/1.0 400 Bad Request\r\n"),
            Status::NotFound => write!(w, "HTTP/1.0 404 Not Found\r\n"),
            Status::InternalServerError => write!(w, "HTTP/1.0 500 Internal Server Error\r\n"),
        };

        for (name, value) in self.headers.iter() {
            write!(w, "{}: ", name)?;
            if let &Some(ref v) = value {
                w.write(v.as_ref())?;
            }
            write!(w, "\r\n")?;
        }

        if !self.headers.contains_key(&"Content-Length".to_string()) {
            if let Some(ref body) = self.body {
                write!(w, "Content-Length: {}", body.len().to_string())?;
            }
        }

        write!(w, "\r\n")?;

        if let Some(ref body) = self.body {
            w.write(body.as_ref())?;
        }

        Ok(())
    }
}

