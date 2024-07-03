# orangutan

orangutan is a lightweight and SUPER simple rust web server library inspired by Python's [Flask](https://flask.palletsprojects.com/en/3.0.x/).

## Installation (cargo)

Use cargo to add orangutan to your project as an dependencie

```bash
cargo add orangutan
```

## Usage

This is what a minimal orangutan application looks like:

```rust
// Use the 'route'-macro to define the path, method(s) and a handler
#[route(path="/hello", method="[POST, GET]")] 
fn hello_handler(request: &Request) -> Response {

    // Handler has to return a Response           
    let mut res = Response::new();

    // Use the 'insert'-method to insert text/json into the Response     
    res.insert("Hello!"); 

    res 

}

fn main() {    
    // Create an new Orangutan
    let mut app = Orangutan::new("127.0.0.1:8080"); 

    // This automatically makes the Orangutan run with the handler and routes assigned to it.      
    app.run();

    // Now you have a web server listening on http://127.0.0.1:8080/hello 
    // Simple, right
} 
```

Output of the program:

```bash
* orangutan being served!
    *  orangutan running on http://127.0.0.1:8080/hello (Press CTRL+C to quit)
```

I recommend using software like [ngrok](https://ngrok.com/) to make the web server public, as there is no way of doing it with orangutan.

# Examples

orangutan is quite a powerful and useful tool. Here are some of the things that orangutan can do!

## Variables in path

```rust
// Same as defining a normal route but with the path containing the type and variable name.
#[route(path="/<str:username>", method="[GET, POST]")]
fn username_handler(request: &Request) -> Response {
    // This handler is responsible for every request in the path: "/something"
    // The name of the variable (username) is defined after the variables type 

    let mut res = Response::new();          

    // We use the get_var() method to get the value of the variable in the request
    let username: String = request.get_var("username");

    // We can do whatever we want with the username
    res.insert(format!("Hello, {}", username));    

    res

}

// Same thing with the username path but now the value of the variable is i32
#[route(path="/<int:number>", method="[POST, GET]")]
fn number_handler(request: &Request) -> Response {
    // This handler is responsible for every request in the path: "/something"
    // Same as username handler except the value of the variable is i32.  

    let mut res = Response::new();          

    let number: i32 = request.get_var("number");

    res.insert(format!("{} * 2 = {}", number, number*2));    

    res

}

fn main() {    
    let mut app = Orangutan::new("127.0.0.1:8080");   

    app.run();     
}
```
## Json Requests

In this example I will show you how to handle Requests that contain Json data

```rust
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

            println!("The order: {}", order.unwrap());
        }
        None => { println!("This means that the request does not contain any json data");}
    }    

    res
}

fn main() {    
    let mut a = Orangutan::new("127.0.0.1:8080");   

    a.run();     
}
```

A great way to test how this works is by utilizing tools like curl on Linux. Lets see how our program behaves.

```bash
curl -i -H "Content-Type: application/json" -X POST -d '{"order":"Computer", "OS": "Linux"}' http://127.0.0.1:8080/order
```

Lets see how out program responses to this:

The output of the program:

```bash
The order: "Computer"
```

What if the request does not contain any json Values?

Lets use this command:

```bash
curl -X GET -H "Content-Type: text/plain" -d "This is not Json Value" http://127.0.0.1:8080/order
```

And again the output is this:

```bash
This means that the request does not contain any json data
```

If the request does contain Json Value but the Value does not contain the "order" field then our program will 'panic' BUT that does not crash the server thankfully.

## Aborting requests

Sometimes you need to simply return an error. This is how:

```rust
#[route(path="/only_vip", method="[GET, POST]")]
fn user_satus_handler(request: &Request) -> Response {

    let mut res = Response::new();          

    let user_data = request.json();
    
    match user_data {        
        Some(data) => { 
            let user_status: Option<&Value> = data.get("user_status");

            if user_status.unwrap() != "VIP" {
                // Use the abort method to create an error for Response. Only 403, 404 and 500 are valid errors for now.
                res.abort(request, 403);
            } else {
                res.insert("Welcome to the club!");
            }            
        }

        None => { res.abort(request, 404) }
    }    

    res
}
```

## Writing Responses

There are many ways of making Responses. Here are some different ways of doing the same thing.

```rust
#[route(path="/", method="[GET, POST]")]
fn handler(request: &Request) -> Response {

    let mut res = Response::new();          

    // Here are 3 methods for doing the same thing
    res.set_content_type(ContentType::TextHtml);
    res.set_status(200);
    res.append("Hello!");

    res.insert("<p>Hello!</p>");

    res.insert("Hello!");
    res.set_content_type(ContentType::TextHtml);

    res
}
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License

[MIT](https://choosealicense.com/licenses/mit/)