use crate::error::RequestPathError;

pub enum RequestPath {
    Push { id: usize, key: u32 },
    Pop { id: usize, key: u32 },
    CloseDown { id: usize, key: u32 },
    CloseUp { id: usize, key: u32 },
    Connect,
    Disconnect { id: usize, key: u32 },
}

impl std::str::FromStr for RequestPath {
    type Err = RequestPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix('/') else {
            return Err(RequestPathError);
        };

        path_with_key(s).or_else(|_| path_connect(s))
    }
}

fn path_with_key(s: &str) -> Result<RequestPath, RequestPathError> {
    let Some((id, s)) = s.split_once('/') else {
        return Err(RequestPathError);
    };
    let id = id.parse().map_err(|_| RequestPathError)?;

    let Some((key, method)) = s.split_once('/') else {
        return Err(RequestPathError);
    };
    let key = key.parse().map_err(|_| RequestPathError)?;

    let p = match method {
        "push" => RequestPath::Push { id, key },
        "pop" => RequestPath::Pop { id, key },
        "close/down" => RequestPath::CloseDown { id, key },
        "close/up" => RequestPath::CloseUp { id, key },
        "disconnect" => RequestPath::Disconnect { id, key },
        _ => return Err(RequestPathError),
    };

    Ok(p)
}

fn path_connect(s: &str) -> Result<RequestPath, RequestPathError> {
    let p = match s {
        "connect" => RequestPath::Connect,
        _ => return Err(RequestPathError),
    };

    Ok(p)
}
