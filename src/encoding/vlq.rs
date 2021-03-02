use std::io;

use sigma_ser::peekable_reader::PeekableReader;

// todo-minor try better: it should somehow define, that vlq is used
// todo-minor maybe move to vlq lib
pub trait TryFromVlq: Sized {
    type Error;

    fn try_from_vlq(data: Vec<u8>) -> Result<Self, Self::Error>;
}

// todo-minor try better: it should somehow define, that vlq is used
// todo-minor maybe move to vlq lib
pub trait TryIntoVlq {
    type Error;

    fn try_into_vlq(&self) -> Result<Vec<u8>, Self::Error>;
}

pub(crate) type DefaultVlqReader<T> = PeekableReader<io::Cursor<T>>;
pub(crate) type DefaultVlqWriter<T> = io::Cursor<T>;

// todo-minor: get_vlq_reader(type, data) - shall be discussed
pub(crate) fn default_vlq_reader<T: AsRef<[u8]>>(data: T) -> DefaultVlqReader<T> {
    PeekableReader::new(io::Cursor::new(data))
}

pub(crate) fn default_vlq_writer<T: AsRef<[u8]>>(data: T) -> DefaultVlqWriter<T> {
    io::Cursor::new(data)
}
