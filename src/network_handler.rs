use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    UNKNOWN,
}

impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            _ => Method::UNKNOWN,
        })
    }
}


#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub body: String,
}

impl Request {
    pub fn parse(data: &[u8]) -> Option<Self> {
        let text = std::str::from_utf8(data).ok()?;

        let mut sections = text.split("\r\n\r\n");

        let headers = sections.next()?;
        let body = sections.next().unwrap_or("");

        let mut first_line = headers.lines().next()?.split_whitespace();

        let method = first_line.next()?.parse().ok()?;
        let path = first_line.next()?.to_string();

        Some(Self {
            method,
            path,
            body: body.to_string(),
        })
    }
}