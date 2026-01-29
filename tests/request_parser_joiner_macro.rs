use http::Request;
use swagger::{request_parser_joiner, RequestParser};

struct TestParser1;

impl RequestParser<()> for TestParser1 {
    fn parse_operation_id(request: &Request<()>) -> Option<&'static str> {
        match request.uri().path() {
            "/test/t11" => Some("t11"),
            "/test/t12" => Some("t12"),
            _ => None,
        }
    }
}

struct TestParser2;

impl RequestParser<()> for TestParser2 {
    fn parse_operation_id(request: &Request<()>) -> Option<&'static str> {
        match request.uri().path() {
            "/test/t21" => Some("t21"),
            "/test/t22" => Some("t22"),
            _ => None,
        }
    }
}

request_parser_joiner!(JoinedReqParser, TestParser1, TestParser2);

#[test]
fn request_parser_joiner_works_from_external_crate() {
    let req1: Request<()> = Request::get("https://www.rust-lang.org/test/t11")
        .body(())
        .unwrap();
    let req2: Request<()> = Request::get("https://www.rust-lang.org/test/t22")
        .body(())
        .unwrap();
    let req3: Request<()> = Request::get("https://www.rust-lang.org/test/t33")
        .body(())
        .unwrap();

    assert_eq!(JoinedReqParser::parse_operation_id(&req1), Some("t11"));
    assert_eq!(JoinedReqParser::parse_operation_id(&req2), Some("t22"));
    assert_eq!(JoinedReqParser::parse_operation_id(&req3), None);
}
