use rflask_macro::route;
use lib_shared::response::*;
use rflask::*;
use lib_shared::request::*;
use serde_json::Value;  

#[route(path="/double/<int:to_dbl>", method="[POST, GET]")]
fn hello_handler(request: &Request) -> Response {

    let mut res = Response::new();      

    let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;

    let v: Value = serde_json::from_str(data).unwrap(); 

    res.insert(v);
    
    res.clear();

    res.insert("<p>Hello, World!</p>");    

    res

}

#[route(path="/double/test", method="[POST, GET]")]
fn double_handler(req: &Request) -> Response {
    let to_dbl: i32 = req.get_var("to_dbl");

    println!("{}", to_dbl);

    /* simpler response generation syntax */
    //utils::make_response(format!("{}", to_dbl * 2), ContentType::TextPlain, 200)

    let mut res = Response::new();

    res.insert("<p>Hello, World!</p>");    

    res
}

fn main() {    

    //let test = Route::new("/<int:test123>", route_shared::Method::POST, hello_handler);

    //println!("{:?}", test);

    let mut a = Comm::new("127.0.0.1:8080");     // todo: eti se route jota käytät sun koodis ja lue se.        

    a.run();     
}