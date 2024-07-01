use std::collections::BTreeMap;
use chrono::Utc;
use serde_json::Value;

use crate::Request;

use crate::request::ContentType;

use crate::utils::{err_403, err_404, err_500};
use crate::is_html;
use crate::type_name;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Default)]
pub struct Response {
    pub status:     u16,
    pub cmsg:       String,
    pub ctype:      String,
    pub headers:    BTreeMap<String, String>,
    pub payload:    Vec<u8>,
}

impl Response {
    pub fn new() -> Response {        
        let mut res = Response {
            status:     200,
            cmsg:       String::from("OK"),
            ctype:      String::from("text/plain"),
            headers:    BTreeMap::new(),
            payload:    Vec::with_capacity(2048),
        };

        let now = Utc::now().format("%a, %d %b %Y, %H:%M:%S %Z").to_string();

        res.add_header("Connection", "close");
        res.add_header("Server", &format!("rustycomms/{}", VERSION));
        res.add_header("Date", &now);

        res
    }    

    /// aborts the Request with an error.
    /// 
    /// Possible error status: 403, 404, 500
    ///
    ///     #[route(path="/test", method="[POST, GET]")]
    ///     fn hello_handler2(request: &Request) -> Response {
    ///
    ///         let mut res = Response::new();                
    ///
    ///         res.abort(request, 404); // when a request is sent to /test it will return the 404 html error.    
    ///
    ///         res
    ///
    ///     }         
    
    pub fn abort(&mut self, req: &Request, status: u16) {
        let response = match status {
            403 => err_403(req),
            404 => err_404(req),
            500 => err_500(req),
            _ => err_404(req),
        };

        self.status = response.status;
        self.cmsg = response.cmsg;
        self.ctype = response.ctype;
        self.headers = response.headers;
        self.payload = response.payload;
    }

    pub fn to_bytes(&self) -> Vec<u8> {        

        let mut response_str = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status,
            self.cmsg
        );
        
        if !self.ctype.is_empty() {
            response_str.push_str(&format!("Content-Type: {}\r\n", self.ctype));
        }

        for (key, value) in &self.headers {
            response_str.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        response_str.push_str("\r\n");

        let mut response_bytes = response_str.into_bytes();

        response_bytes.extend_from_slice(&self.payload);

        response_bytes
    }

    /// a way to add a header to a Response

    pub fn add_header(&mut self, key: &str, value: &str) {
        if !self.headers.contains_key(key) {
            self.headers.insert(String::from(key), String::from(value));
        }
    }

    /// a way to add a status to a Response

    pub fn set_status(&mut self, status: u16) {
        self.status = status;
        self.cmsg = Response::get_http_message(status);
    }

    /// a way to add set content type to a Response. Must use the ContentType enum.
    /// 
    ///         pub fn set_content_type(&mut self, ctype: ContentType) {          
    ///             let ctype_str = match ctype {
    ///                 ContentType::ApplicationJson => "application/json",
    ///                 ContentType::TextPlain => "text/plain",
    ///                 ContentType::TextHtml => "text/html"
    ///             };        
    ///
    ///             self.ctype = String::from(ctype_str);
    ///         }                    
    
    pub fn set_content_type(&mut self, ctype: ContentType) {          
        let ctype_str = match ctype {
            ContentType::ApplicationJson => "application/json",
            ContentType::TextPlain => "text/plain",
            ContentType::TextHtml => "text/html"
        };        

        self.ctype = String::from(ctype_str);
    }     

    /// clears the Response
    /// 
    ///         #[route(path="/", method="[POST, GET]")]
    ///         fn hello_handler(request: &Request) -> Response {
    ///
    ///             let mut res = Response::new();        
    ///
    ///             res.insert("<p>Hello, World!</p>");             
    ///
    ///             res.clear(); // clears the Response making it the same as Response::new()
    ///
    ///             res
    /// 
    ///         }

    pub fn clear(&mut self) {
        self.status = 200;
        self.cmsg = String::from("OK");
        self.ctype = String::from("text/plain");
        self.headers = BTreeMap::new();
        self.payload = Vec::with_capacity(2048);
    }

    /// A simple way to make a Response have some text/json/html in it.
    /// 
    /// Can be used with string that contains html syntax.
    /// 
    /// The types of values handled given as parameters:
    /// 
    ///     serde_json::value::Value
    ///     &str
    ///     String
    ///     
    /// Sets the content-type, cmsg and status to default values.
    /// 
    /// Example:
    /// 
    ///         #[route(path="/", method="[POST, GET]")]
    ///         fn hello_handler(request: &Request) -> Response {
    ///
    ///             let mut res = Response::new();        
    ///
    ///             res.insert("<p>Hello, World!</p>"); // Now the Response contains "<p>Hello, World!</p>" as payload              
    ///
    ///             res
    ///         }    
    /// 
    
    pub fn insert<T: ToOutput + ToString + std::fmt::Debug>(&mut self, payload: T) {
        self.set_status(200);              
        self.cmsg = String::from("OK");   

        match type_of(&payload) {
            "serde_json::value::Value" => { self.set_content_type(ContentType::ApplicationJson); },            
            "&str" | "String" => { if is_html(payload.to_string()) { self.set_content_type(ContentType::TextHtml); } else { self.set_content_type(ContentType::TextPlain); }},
            _ => { if is_html(payload.to_string()) { self.set_content_type(ContentType::TextHtml); } else { self.set_content_type(ContentType::TextPlain); }},
        }                

        self.append(payload);                         
    }    

    pub fn append<T: ToOutput>(&mut self, payload: T) {
        self.payload.extend(payload.to_output().iter());
    }

    fn get_http_message(status: u16) -> String {
        let msg = match status {
            100 => "Continue",
            101 => "Switching Protocols",
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            203 => "Non-Authoritative Information",
            204 => "No Content",
            205 => "Reset Content",
            206 => "Partial Content",
            300 => "Multiple Choices",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            305 => "Use Proxy",
            307 => "Temporary Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            402 => "Payment Required",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            407 => "Proxy Authentication Required",
            408 => "Request Time Out",
            409 => "Conflict",
            410 => "Gone",
            411 => "Length Required",
            412 => "Precondition Failed",
            413 => "Request Entity Too Large",
            414 => "Request-URI Too Large",
            415 => "Unsupported Media Type",
            416 => "Requested Range Not Satisfiable",
            417 => "Expectation Failed",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Time-out",
            505 => "HTTP Version Not Supported",
            _   => "OK",
        };

        String::from(msg)
    }
}

fn type_of<T>(_: &T) -> &str {
    type_name::<T>()
}

pub trait ToOutput {
    fn to_output(&self) -> Vec<u8>;
}

impl ToOutput for Value {
    fn to_output(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }    
}

impl ToOutput for &str {
    fn to_output(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl ToOutput for String {
    fn to_output(&self) -> Vec<u8> { 
        self.as_bytes().to_vec()        
    }
}