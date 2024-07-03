use std::{any::type_name, sync::Mutex};
use lazy_static::lazy_static;
use regex::Regex;

use crate::response::Response;
use crate::request::Request;

pub mod request;
pub mod utils;
pub mod response;

const HTML_TAGS: [&str; 117] = [
    "a",
    "abbr",
    "address",
    "area",
    "article",
    "aside",
    "audio",
    "b",
    "base",
    "bdi",
    "bdo",
    "blockquote",
    "body",
    "br",
    "button",
    "canvas",
    "caption",
    "cite",
    "code",
    "col",
    "colgroup",
    "data",
    "datalist",
    "dd",
    "del",
    "details",
    "dfn",
    "dialog",
    "div",
    "dl",
    "dt",
    "em",
    "embed",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "i",
    "iframe",
    "img",
    "input",
    "ins",
    "kbd",
    "label",
    "legend",
    "li",
    "link",
    "main",
    "map",
    "mark",
    "math",
    "menu",
    "menuitem",
    "meta",
    "meter",
    "nav",
    "noscript",
    "object",
    "ol",
    "optgroup",
    "option",
    "output",
    "p",
    "param",
    "picture",
    "pre",
    "progress",
    "q",
    "rb",
    "rp",
    "rt",
    "rtc",
    "ruby",
    "s",
    "samp",
    "script",
    "section",
    "select",
    "slot",
    "small",
    "source",
    "span",
    "strong",
    "style",
    "sub",
    "summary",
    "sup",
    "svg",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "time",
    "title",
    "tr",
    "track",
    "u",
    "ul",
    "var",
    "video",
    "wbr", 
];


/// Used to check if the Response returned by a handler is Html 
pub fn is_html(payload: String) -> bool {    
    let re = Regex::new(r"\s?<!doctype html>|(<html\b[^>]*>|<body\b[^>]*>|<x-[^>]+>)+").unwrap();
    let re_full_str = HTML_TAGS.map(|x| format!("<{}\\b[^>]*>", x)).join("|");
    let re_full = Regex::new(re_full_str.as_str()).unwrap();
    re.is_match(&payload) || re_full.is_match(&payload)
}

/// contains some important info about the Route. Arguably useless
pub struct RouteInfo {    
    pub path: String,
    pub methods: Vec<String>,    
    pub handler: fn(&Request) -> Response,    
}

impl RouteInfo {
    pub fn new(path: String, methods: Vec<String>, handler: fn(&Request) -> Response) -> Self {        
        RouteInfo {
            path,
            methods,                    
            handler,            
        }
    }
}

// Creates ROUTES, where the Routes are stored. "Must" be static so that it can be accessed from elsewhere code.
// Might just simply be bad code but I could not figure out anything better...
lazy_static! {    
    pub static ref ROUTES: Mutex<Vec<RouteInfo>> = Mutex::new(Vec::new()); 
}
/// Pushes the given route to the ROUTES
pub fn add_route(route: RouteInfo) {        
    ROUTES.lock().unwrap().push(route);    
}

