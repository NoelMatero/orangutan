use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::net::ToSocketAddrs;

use lib_shared::{RouteInfo,utils, ROUTES};
use lib_shared::response::Response;
use lib_shared::request::{match_method, Method, Request};


use threadpool::ThreadPool;
use mio::util::Slab;
use mio::tcp::{TcpStream, TcpListener};
use mio::{Token, EventSet, EventLoop, PollOpt, Sender, Handler, TryRead};

pub mod route;

struct Client {
    sock:   TcpStream,
    token:  Token,
    events: EventSet,
    i_buf:  Vec<u8>,
    o_buf:  Vec<u8>,
}

impl Client {
    /// Creates a new Client instance with an initial capacity for input and output buffers.
    fn new(sock: TcpStream, token: Token) -> Client {
        Client {
            sock,
            token,
            events: EventSet::hup(),
            i_buf:  Vec::with_capacity(2048),
            o_buf:  Vec::new(),
        }
    }

    /// Reads data from the socket into the input buffer. 
    /// Returns Ok(true) if data was read successfully, otherwise Ok(false) or an Err.
    fn receive(&mut self) -> Result<bool, std::io::Error> {
        let mut bytes_read: usize = 0;

        loop {
            let mut buf: Vec<u8> = Vec::with_capacity(2048);
            match self.sock.try_read_buf(&mut buf) {
                Ok(size)  => {
                    match size {
                        Some(bytes) => {
                            self.i_buf.extend(buf);
                            bytes_read += bytes;
                        },
                        None    => {
                            self.events.remove(EventSet::readable());
                            self.events.insert(EventSet::writable());
                            break;
                        },
                    }
                },
                Err(_)  => {
                    self.events.remove(EventSet::readable());
                    self.events.insert(EventSet::writable());
                    break;
                },
            };
        }

        Ok(bytes_read > 0)
    }

    /// Writes data from the output buffer to the socket.
    /// Returns Ok(true) if data was sent successfully, Ok(false) if there's no data to send, or an Err.
    fn send(&mut self) -> Result<bool, std::io::Error> {
        if self.o_buf.is_empty() {
            return Ok(false);
        }

        while !self.o_buf.is_empty() {
            match self.sock.write(&self.o_buf.as_slice()) {
                Ok(sz)  => {
                    if sz == self.o_buf.len() {
                        // we did it!
                        self.events.remove(EventSet::writable());
                        break;
                    } else {
                        // keep going
                        self.o_buf = self.o_buf.split_off(sz);
                    }
                },
                Err(_)  => {
                    return Ok(true);
                }
            }
        }

        Ok(true)
    }

    /// Registers the client with the event loop to listen for events.
    /// Adds the `readable` event to the client's event set.
    fn register(&mut self, evl: &mut EventLoop<Comm>) -> Result<(), std::io::Error> {
        self.events.insert(EventSet::readable());
        evl.register(&self.sock, self.token, self.events, PollOpt::edge() | PollOpt::oneshot())
    }

    /// Re-registers the client with the event loop to continue listening for events.
    fn reregister(&mut self, evl: &mut EventLoop<Comm>) -> Result<(), std::io::Error> {
        evl.reregister(&self.sock, self.token, self.events, PollOpt::edge() | PollOpt::oneshot())
    }
}

pub struct Comm {
    routes:  HashMap<route::RouteDef, route::Route>,
    rcache:  HashMap<route::RouteDef, route::RouteDef>,
    server:  Option<TcpListener>,
    token:   Token,        
    conns:   Slab<Client>,
    default: fn(&Request) -> Response,
    tpool:   ThreadPool,
}

impl Handler for Comm {
    type Timeout = ();
    type Message = (Token, Vec<u8>);    

    /// Handles events for the event loop.
    /// Determines if the event is for the server socket (new connection) or an existing client (read/write).
    fn ready(&mut self, evl: &mut EventLoop<Comm>, token: Token, events: EventSet) {
        if events.is_error() || events.is_hup() {
            self.reset_connection(token);
            return;
        }

        if events.is_readable() {
            if self.token == token {
                // New connection on the server socket
                let sock = self.accept().unwrap();

                if let Some(token) = self.conns.insert_with(|token| Client::new(sock, token)) {
                    self.get_client(token).register(evl).ok();
                }
                self.reregister(evl);

            } else {
                // Read data from existing connection
                self.readable(evl, token)
                    .and_then(|_| self.get_client(token)
                                      .reregister(evl)).ok();
            }

            return;
        }

        if events.is_writable() {
            match self.get_client(token).send() {
                Ok(true)    => { self.reset_connection(token); },
                Ok(false)   => { let _ = self.get_client(token).reregister(evl); },
                Err(_)      => {},
            }
        }
    }

    fn notify(&mut self, evl: &mut EventLoop<Comm>, msg: (Token, Vec<u8>)) {
        let (token, output) = msg;
        let client = self.get_client(token);

        client.o_buf = output;
        let _ = client.reregister(evl);
    }
}

impl Comm {
    pub fn new<A: ToSocketAddrs>(address: A) -> Self {  

        let server = Some(TcpListener::bind(&address.to_socket_addrs().unwrap().next().unwrap()).unwrap());           

        Comm {
            routes:  HashMap::new(), 
            rcache:  HashMap::new(), 
            server,
            token:   Token(1),         
            conns:   Slab::new_starting_at(Token(2), 2048),  
            default: utils::err_404,
            tpool:   ThreadPool::new(255),            
        }
    }

    fn accept(&mut self) -> Result<TcpStream, std::io::Error> {
        if let Some(ref server) = self.server {
            if let Ok(s) = server.accept() {
                if let Some((sock, _)) = s {
                    return Ok(sock);
                }
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionAborted,
            "connection aborted prematurely".to_string()
        ))
    }

    fn readable(&mut self, evl: &mut EventLoop<Comm>, token: Token) -> Result<bool, std::io::Error> {
        if let Ok(true) = self.get_client(token).receive() {
            let buf = self.get_client(token).i_buf.clone();
            if let Ok(rqstr) = String::from_utf8(buf) {
                self.handle_request(token, evl.channel(), &rqstr);
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn reregister(&mut self, evl: &mut EventLoop<Comm>) {
        if let Some(ref server) = self.server {
            evl.reregister(server, self.token,
                                 EventSet::readable(),
                                 PollOpt::edge() | PollOpt::oneshot()).ok();
        }
    }

    pub fn run(&mut self) {        
        let routes: std::sync::MutexGuard<Vec<RouteInfo>> = ROUTES.lock().unwrap();           

        let mut path = String::new();        

        let _a = path; // gets rid of the warning
        
        let mut handler: fn(&Request) -> Response = self.default;

        let _b = handler; // gets rid of the warning

        let mut methods: HashSet<Method> = HashSet::new();

        for route in routes.iter() {            
            path = route.path.clone(); 

            handler = route.handler; 

            for method in route.methods.iter() {
                methods.insert(match_method(method));
            }

            for m in &methods {                
                let routedef = route::RouteDef {
                    path: path.clone(),
                    method: *m,
                };
    
                if self.routes.contains_key(&routedef) {
                    panic!("Route handler for {} has already been defined!", path); 
                }
    
                self.routes.insert(routedef, route::Route::new(&path, *m, handler));
            }
        }                               

        let mut evl = match EventLoop::new() {
            Ok(event_loop)  => event_loop,
            Err(_)          => panic!("unable to initiate event loop"),
        };

        match self.server {
            None    => println!("server not bound to an address!"),
            Some(_) => {
                println!("Server being run on {:?}", self.server.as_mut().unwrap().local_addr().unwrap());       
                self.register(&mut evl).ok();                
                evl.run(self).unwrap();                         
            },
        };                 
    }

    fn register(&mut self, evl: &mut EventLoop<Comm>) -> Result<(), std::io::Error> {
        if let Some(ref server) = self.server {            
            return evl.register(server, self.token, EventSet::readable(), PollOpt::edge() | PollOpt::oneshot());
        }

        Ok(())
    }

    fn reset_connection(&mut self, token: Token) {     
        self.conns.remove(token);
    }

    fn handle_request(&mut self, token: Token, tx: Sender<(Token, Vec<u8>)>, rqstr: &str) {   
        let mut request = match rqstr.parse::<Request>() {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to parse request: {:?}", e);
                return;
            }
        };

        let mut handler: fn(&Request) -> Response = self.default;        

        let routedef = route::RouteDef {
            path: request.path.clone(),
            method:  request.method,
        };        

        if self.rcache.contains_key(&routedef) {
            let route = &self.routes[&self.rcache[&routedef]];

            handler = route.handler;
            request.params = route.parse(&request.path);
        } else {            
            for (path, route) in &self.routes {                

                if route.is_match(&request) {                
                    handler = route.handler;
                    request.params = route.parse(&request.path);
                    self.rcache.insert(routedef.clone(), (*path).clone());
                    break;
                }
            }
        }        

        self.tpool.execute(move || {
            let _ = tx.send((token, handler(&request).to_bytes()));
        });        
    }

    fn get_client(&mut self, token: Token) -> &mut Client {        
        self.conns.get_mut(token).unwrap()
    }
}