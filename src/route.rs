use std::collections::HashMap;
use regex::Regex;

use lib_shared::request::{Method, Request};

use lib_shared::response::Response;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ParamType { 
    Integer,
    Unsigned,
    String,
    Float,
    Path,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct RouteDef {
    pub path: String,
    pub method:  Method,
}

#[derive(Debug)]
pub struct Route {
    matcher:     Regex, 
    pub method:      Method,
    pub params:      HashMap<String, ParamType>,
    pub handler: fn(&Request) -> Response,
}


impl Route {
    pub fn new(path: &str, method: Method, 
        handler: fn(&Request) -> Response) -> Self { 

        let re = Regex::new(r"^<(?:(int|uint|str|float|path):)?([\w_][a-zA-Z0-9_]*)>$").unwrap();

        let parts: Vec<&str> = path.split('/').filter(|&s| s != "").collect();        

        let mut matcher: String = String::from(r"^");

        let mut params: HashMap<String, ParamType> = HashMap::new();

        for part in parts {
            let chunk: String = if re.is_match(part) {
                let caps = re.captures(part).unwrap();
                let param = caps.get(2).unwrap().as_str();
                let ptype: ParamType = match caps.get(1) {
                    Some(x)     => {
                        match x.as_str() {
                            "int"   => ParamType::Integer,
                            "uint"  => ParamType::Unsigned,
                            "float" => ParamType::Float,
                            "path"  => ParamType::Path,
                            "str"   => ParamType::String,
                            _       => ParamType::String,
                        }
                    }
                    None        => ParamType::String,
                };

                let mstr: String = match ptype {
                    ParamType::String   => String::from(r"(?:[^/])+"),
                    ParamType::Integer  => String::from(r"-*[0-9]+"),
                    ParamType::Unsigned => String::from(r"[0-9]+"),
                    ParamType::Float    => String::from(r"-*[0-9]*[.]?[0-9]+"),
                    ParamType::Path     => String::from(r".+"),
                };

                params.insert(String::from(param), ptype);

                format!("/(?P<{}>{})", &param, &mstr)
            } else {
                String::from("/") + part
            };

            matcher.push_str(&chunk);
        }

        matcher.push_str("/?$");                   

        Route {
            matcher: Regex::new(&matcher).unwrap(),
            params,
            method,
            handler,
        }        
    }

    pub fn parse(&self, path: &str) -> HashMap<String, String> {
        let mut params: HashMap<String, String> = HashMap::new();

        if self.matcher.is_match(&path) {
            let caps = self.matcher.captures(path).unwrap();
            for param in self.params.keys() {
                params.insert(param.clone(), String::from(caps.name(&param).unwrap().as_str()));
            }
        }

        params
    }

    pub fn is_match(&self, req: &Request) -> bool {
        self.matcher.is_match(&req.path) && self.method == req.method
    }
}