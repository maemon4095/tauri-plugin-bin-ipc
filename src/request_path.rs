use crate::error::RequestPathError;

pub struct RequestPath {
    pub ty: RequestType,
    pub id: usize,
    pub key: u32,
}

pub enum RequestType {
    Push,
    Pop,
    CloseDown,
    CloseUp,
    Connect,
}

impl std::str::FromStr for RequestPath {
    type Err = RequestPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix('/') else {
            return Err(RequestPathError);
        };

        let Some((id, s)) = s.split_once('/') else {
            return Err(RequestPathError);
        };

        let Some((key, method)) = s.split_once('/') else {
            return Err(RequestPathError);
        };

        let id = id.parse().map_err(|_| RequestPathError)?;
        let key = key.parse().map_err(|_| RequestPathError)?;
        let ty = match method {
            "push" => RequestType::Push,
            "pop" => RequestType::Pop,
            "close/down" => RequestType::CloseDown,
            "close/up" => RequestType::CloseUp,
            "connect" => RequestType::Connect,
            _ => return Err(RequestPathError),
        };

        Ok(Self { ty, id, key })
    }
}
