#[derive(Debug)]
pub struct BinIpcError {
    is_reportable: bool,
    boxed: Box<dyn std::error::Error + Send>,
}

impl BinIpcError {
    pub fn into_inner(self) -> Box<dyn std::error::Error + Send> {
        return self.boxed;
    }

    pub fn is_reportable(&self) -> bool {
        self.is_reportable
    }

    pub fn new_reportable<E: 'static + std::error::Error + Send>(err: E) -> Self {
        Self {
            is_reportable: true,
            boxed: Box::new(err),
        }
    }
}

impl<E: 'static + std::error::Error + Send> From<E> for BinIpcError {
    fn from(value: E) -> Self {
        Self {
            is_reportable: false,
            boxed: Box::new(value),
        }
    }
}
