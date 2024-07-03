use crate::response::{Response, ToOutput};
use crate::Request;
use crate::request::ContentType;

pub fn make_response<T: ToOutput>(body: T, c_type: ContentType, status: u16) -> Response {
    let mut res = Response::new();

    res.set_status(status);
    res.set_content_type(c_type);
    res.append(body);

    res
}

pub fn err_body(message: &str, path: &str) -> String {
    format!("<html><head>\
             <style>body {{ font-family: helvetica, sans-serif; }} p {{ font-size: 14 }}</style>\
             </head><body><h3>Your request failed</h3><p>{}: {}</p></body></html>", message, path)
}

/// Default handler function for HTTP 403 errors.
pub fn err_403(req: &Request) -> Response {
    make_response(err_body("forbidden", &req.path), ContentType::TextHtml, 403)
}

/// Default handler function for HTTP 403 errors for XHR.
pub fn err_403_json(message: &str) -> Response {
    make_response(format!("{{ message: 'forbidden: {}' }}", message), ContentType::ApplicationJson, 403)
}

/// Default handler function for HTTP 404 errors.
pub fn err_404(req: &Request) -> Response {
    make_response(err_body("not found", &req.path), ContentType::TextHtml, 404) // tee tost kunno
}

/// Default handler function for HTTP 500 errors for XHR.
pub fn err_404_json(message: &str) -> Response {
    make_response(format!("{{ message: 'not found: {}' }}", message), ContentType::ApplicationJson, 404)
}

/// Default handler function for HTTP 500 errors.
pub fn err_500(req: &Request) -> Response {
    make_response(err_body("internal server error", &req.path), ContentType::TextHtml, 500)
}

/// Default handler function for HTTP 500 errors for XHR.
pub fn err_500_json(message: &str) -> Response {
    make_response(format!("{{ message: 'internal server error: {}' }}", message), ContentType::ApplicationJson, 500)
}
