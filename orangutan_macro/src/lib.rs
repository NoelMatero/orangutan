use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

/// The route macro used to define the path and the method for a handler.
/// 
/// Example of a route macro:
/// 
///     #[route(path="/", method="[POST]")]
/// 
/// Here the path is "/" and methods are POST and GET
/// 
/// Full example:
/// 
///     #[route(path="/hello", method="[POST, GET]")]
///     fn hello_handler(request: &Request) -> Response {
///     
///         let mut res = Response::new();        
///     
///         res.insert("<p>Hello, World!</p>");           
///     
///         res
///     }
/// 
/// Here the path is "/hello" and the hello_handler function is responsible for handling any requests sent to the path.
/// 

#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    let handler = parse_macro_input!(item as ItemFn);
    let handler_name = &handler.sig.ident;

    let args = attr.to_string();

    let mut path = String::new();
    let mut methods: Vec<String> = Vec::new();    

    let parts: Vec<&str> = args.split(',').collect();

    // following code reads the path and methods

    for part in parts {
        let trimmed = part.trim();
        if trimmed.starts_with("path") {
            if let Some((_, value)) = trimmed.split_once('=') {
                path = value.trim_matches(|c| c == '"' || c == ' ').to_string();                
            }
        } else if trimmed.starts_with("method") {
            if let Some((_, value)) = trimmed.split_once('=') {
                let method_str = value.trim_matches(|c| c == '[' || c == ']' || c == '"');
                methods = method_str.split(',')
                                    .map(|s| s.trim_matches(|c| c == '"' || c == ' ' || c == '[' || c == ']').to_string())
                                    .collect();
            }
        } else {
            let method_str = trimmed.trim_matches(|c| c == '[' || c == ']' || c == '"');
            methods.push(method_str.to_string());
        }
    }

    // checks if the methods are of valid type

    for method in methods.clone() {
        match method.as_str() {
            "GET" | "POST" | "PUT" | "DELETE" | "OPTIONS" => {},
            _ => panic!("You are using method: {}, which is not a valid method", method),
        }
    }

    // Use the handler name to create a unique module name
    let module_name = format_ident!("route_{}", handler_name);     

    let expanded = quote! {
        #handler        

        #[allow(non_snake_case)]
        mod #module_name {
            use super::*;
            use lib_shared::RouteInfo;
            use lib_shared::response::Response;
            use lib_shared::request::Request;
            use lib_shared::*;
            use ctor::ctor;            

            #[ctor]
            fn register_route() { // adds the defined ruote to ROUTES where it can be read from elsewhere of the code 
                let route_info = RouteInfo::new(
                    #path.to_string(),
                    vec![#(#methods.to_string()),*],
                    #handler_name as fn(&Request) -> Response,                                        
                );
                add_route(route_info);
            }
        }
    };

    TokenStream::from(expanded)
}