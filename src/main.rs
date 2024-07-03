use orangutan_macro::route;
use lib_shared::response::{self, *};
use orangutan::*;
use lib_shared::request::*;
use serde_json::Value;

#[route(path="/", method="[GET, POST]")]
fn handler(request: &Request) -> Response {

    let mut res = Response::new();          

    // Here are 3 methods for doing the same thing
    res.set_content_type(ContentType::TextHtml);
    res.set_status(200);
    res.append("Hello!");

    res.insert("<p>Hello, World!</p>");

    res.insert("Hello!");
    res.set_content_type(ContentType::TextHtml);    

    res
}

#[route(path="/order", method="[GET, POST]")]
fn order_handler(request: &Request) -> Response {

    let mut res = Response::new();          

    // .json() returns an Option<Value>, where Some represents the json Value and None means that there is no json Value in the request
    let order_data = request.json();

    // Let us use match to see if the request contains Json Value or not
    match order_data {
        // Great. The reqeust contains Json Value
        Some(data) => { 
            let order: Option<&Value> = data.get("order");

            println!("The order{}", order.unwrap());
        }
        None => { println!("This means that the request does not contain any json data");}
    }    

    res
}

#[route(path="/number/<int:number>", method="[POST, GET]")]
fn number_handler(request: &Request) -> Response {

    let mut res = Response::new();          

    let number: i32 = request.get_var("number");

    res.insert(format!("{} * 2 = {}", number, number*2));    

    res

}

fn main() {    

    let mut a = Orangutan::new("127.0.0.1:8080");   

    a.run();     
}