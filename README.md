# rFlask

rFlask is a lightweight and SUPER simple rust web server library inspired by Python's [Flask](https://flask.palletsprojects.com/en/3.0.x/).

## Installation (cargo)

Use cargo to add rFlask to your project as an dependencie

```bash
cargo add rflask
```

## Usage

This is what a minimal rFlask application looks like:

```rust
#[route(path="/hello", method="[POST, GET]")] // Use the 'route'-macro to define the path, method(s) and handler
fn hello_handler(request: &Request) -> Response {

    let mut res = Response::new(); // Handler has to return a Response           

    res.insert("Hello!"); // Use the 'insert'-method to insert text/json into the Response     

    res 

}

fn main() {    

    let mut a = App::new("127.0.0.1:8080"); // Create a new App

    a.run(); // This automatically makes the App run with the handler and routes assigned to it.      
    
} // Simple, right
```