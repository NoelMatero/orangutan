use rflask_macro::route;
use lib_shared::response::*;
use rflask::*;
use lib_shared::request::*;

#[route(path="/hello", method="[POST, GET]")]
fn hello_handler(request: &Request) -> Response {

    let mut res = Response::new();          

    res.insert("Hello!");    

    res

}

fn main() {    

    let mut a = App::new("127.0.0.1:8080");   

    a.run();     
}