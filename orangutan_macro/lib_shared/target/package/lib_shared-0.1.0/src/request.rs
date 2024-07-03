use std::collections::HashMap;
use serde_json::Value;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum ContentType {
    ApplicationJson,
    TextPlain,
    TextHtml,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum QueryArg {
    Single(String),
    Multiple(Vec<String>),    
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Method {
    GET,    
    POST,
    PUT,
    DELETE,
    OPTIONS,       
    NONE, 
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub uri: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub params: HashMap<String, String>,
}

impl Request {

    /// A way to create a new empty Request
    /// 
    ///     Some important fields:
    /// 
    ///  method: Represents the http method 
    ///  path: Represents the path that the Request uses
    ///  headers: Represents the headers of a Request
    ///  body: The body/data of the Request. Containts the "data"
    ///  params: The params of the Request. Example: if the path is "/<int:test>", then the params are the int variable and the naem of the variable is test.    

    pub fn new() -> Request {
        Request {
            method: Method::NONE,
            uri: String::new(),
            path: String::new(),
            query: None,
            headers: HashMap::new(),
            body: Vec::new(),    
            params: HashMap::new(),
        }
    }        

    /// returns Request with the params being inserted to the Request's params field 

    pub fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.params = params;
        self
    }

    /// Pulls the variable from the route, if the route is defined with params. 
    /// 
    /// Example:  
    /// 
    ///     #[route(path="/<int:name_defined_here>", method="[POST, GET]")]
    ///     fn double_handler(req: &Request) -> Response {
    ///         let pulled_variable: i32 = req.get_var("not_the_same_name"); // This panics because:
    ///         // the path defined in the route_macro is NOT the same as the one given to the get_var() function
    ///
    ///         let mut res = Response::new();              
    ///
    ///         res
    ///     }    

    pub fn get_var<T: FromUri>(&self, name: &str) -> T {
        if !self.params.contains_key(name) {
            panic!("Invalid route parameter {:?}", name);
        }

        FromUri::from_uri(&self.params[name])
    }

    /// returns Request in json.
    /// 
    /// Returns Option<Value>, where:
    /// 
    /// Some: Is the Request in json. 
    /// 
    /// None: The request is not json.

    pub fn json(&self) -> Option<Value> { // todo checkkaa toimiiks muilki serde json jutuils, jotka ei oo Value

        let binding = self.body.clone();
        let data = std::str::from_utf8(&binding).ok();

        match data {
            Some(data) =>{  let v: Option<Value> = serde_json::from_str(data).ok(); return v; },
            None => { let v: Option<Value> = serde_json::from_str("").ok(); return v; },
        }            
    }

    /// returns the Request's body as string

    pub fn get_string(&self) -> String {
        return String::from_utf8(self.body.clone()).expect("Bytes should be valid utf8");
    }    

    pub fn parse(&mut self, rqstr: &str) -> std::result::Result<(), RequestError> {
        let mut lines = rqstr.split("\r\n");
                
        if let Some(request_line) = lines.next() {
            let parts: Vec<&str> = request_line.splitn(3, ' ').collect();
            if parts.len() != 3 {
                return Err(RequestError::InvalidRequestLine);
            }
            self.method = match_method(parts[0]);
            self.uri = parts[1].to_string();
            self.parse_uri(parts[1]);
        } else {
            return Err(RequestError::InvalidRequestLine);
        }
        
        for line in lines.by_ref() {
            if line.is_empty() {
                break; 
            }

            if let Some((name, value)) = line.split_once(": ") {
                self.headers.insert(name.to_lowercase(), value.to_string());
            }
        }
        
        self.body = lines.collect::<Vec<&str>>().join("\r\n").into_bytes();

        Ok(())
    }

    fn parse_uri(&mut self, uri: &str) {
        if let Some((path, query)) = uri.split_once('?') {
            self.path = path.to_string();
            self.query = Some(query.to_string());
        } else {
            self.path = uri.to_string();
            self.query = None;
        }
    }
}

pub fn match_method(method: &str) -> Method {
    match method {
        "GET" => Method::GET, 
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "OPTIONS" => Method::OPTIONS,        
        _ => { Method::NONE },
    }
}

#[derive(Debug)]
pub enum RequestError {
    JsonStrError(serde_json::Error),
    StrCopyError(std::string::FromUtf8Error),
    InvalidRequestLine,
}

impl std::str::FromStr for Request {
    type Err = RequestError;
    
    fn from_str(rqstr: &str) -> std::result::Result<Request, RequestError> {
        let mut req = Request::new();
        req.parse(rqstr).unwrap();
        Ok(req)
    }
}

pub trait FromUri {
    fn from_uri(data: &str) -> Self;
}

impl FromUri for String {
    fn from_uri(data: &str) -> String {
        String::from(data)
    }
}

impl FromUri for i32 {
    fn from_uri(data: &str) -> i32 {
        data.parse::<i32>().expect("matched integer can't be parsed")
    }
}

impl FromUri for u32 {
    fn from_uri(data: &str) -> u32 {
        data.parse::<u32>().expect("matched integer can't be parsed")
    }
}

impl FromUri for f32 {
    fn from_uri(data: &str) -> f32 {
        data.parse::<f32>().expect("matched float can't be parsed")
    }
}