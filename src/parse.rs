use std::str;

use crate::*;

/// A stateful wrapper to make a `Str` a readable `Source`
pub struct Cursor<S> {
    offset: usize,
    source: S,
}

impl<'de, S: Str<'de>> Cursor<S> {
    /// Creates a `Cursor` that reads the whole `Str`
    pub fn new(source: S) -> Self {
        Cursor { offset: 0, source }
    }
}

impl<'de, S: Str<'de>> Source<'de, S> for Cursor<S> {
    fn offset(&self) -> usize {
        self.offset
    }

    fn limit(&self) -> usize {
        self.source.len()
    }

    fn read_u8_char(&mut self) -> IoResult<u8> {
        if self.offset + 1 >= self.source.len() {
            return Err(Error::UnexpectedEof.into());
        }
        match unsafe { self.source.get_u8_char(self.offset) } {
            Some(char) => {
                self.offset += 1;
                Ok(char)
            }
            None => Err(Error::BadEncoding(self.offset).into()),
        }
    }

    fn read_str(&mut self, n: usize) -> IoResult<S> {
        let j = self.offset + n;
        if j >= self.source.len() {
            return Err(Error::UnexpectedEof.into());
        }
        match unsafe { self.source.clone_slice(self.offset, j) } {
            Some(s) => {
                self.offset = j;
                Ok(s)
            }
            None => Err(Error::BadEncoding(self.offset).into()),
        }
    }

    unsafe fn read_until(&mut self, byte: u8) -> IoResult<S> {
        let offset = match self.source.find(self.offset, byte) {
            Some(offset) => offset,
            None => return Err(Error::UnexpectedEof.into()),
        };
        self.read_str(offset - self.offset)
    }
}

impl<'de, S: Str<'de>> Value<S> {
    /// Parses a string or byte array
    pub fn parse(source: S) -> IoResult<Self> {
        let cursor = Cursor { offset: 0, source };
        Self::from_source(cursor)
    }

    /// Parses a stream
    pub fn from_source(mut source: impl Source<'de, S>) -> IoResult<Self> {
        match source.read_u8_char()? {
            b'N' => read_null(source),
            b'b' => read_bool(source),
            b'i' => read_int(source),
            b'd' => read_float(source),
            b's' => read_string(source),
            b'a' => read_array(source),
            b'O' => read_object(source),
            b'C' => read_ser(source),
            b'R' => read_ref(source),
            _ => Err(Error::BadToken(source.offset()).into()),
        }
    }
}

fn read_null<'de, S: Str<'de>>(mut source: impl Source<'de, S>) -> IoResult<Value<S>> {
    expect_char(&mut source, b';')?;
    Ok(Value::Null)
}

fn read_bool<'de, S: Str<'de>>(mut source: impl Source<'de, S>) -> IoResult<Value<S>> {
    expect_char(&mut source, b':')?;
    let bool = match source.read_u8_char()? {
        b'1' => true,
        b'0' => false,
        _ => return Err(Error::BadNumber(source.offset()).into()),
    };
    expect_char(&mut source, b';')?;
    Ok(Value::Bool(bool))
}

fn read_int<'de, S: Str<'de>>(mut source: impl Source<'de, S>) -> IoResult<Value<S>> {
    expect_char(&mut source, b':')?;
    Ok(Value::Int(parse_before::<'_, i64, _, _>(
        &mut source,
        b';',
    )?))
}

fn read_float<'de, S: Str<'de>>(mut source: impl Source<'de, S>) -> IoResult<Value<S>> {
    expect_char(&mut source, b':')?;
    Ok(Value::Float(parse_before::<'_, f64, _, _>(
        &mut source,
        b';',
    )?))
}

fn read_string<'de, S: Str<'de>>(mut source: impl Source<'de, S>) -> IoResult<Value<S>> {
    expect_char(&mut source, b':')?;
    let len = parse_before::<'_, usize, _, _>(&mut source, b':')?;
    expect_char(&mut source, b'"')?;
    let content = source.read_str(len)?;
    expect_char(&mut source, b'"')?;
    expect_char(&mut source, b';')?;
    Ok(Value::String(content))
}

fn read_array<'de, S: Str<'de>>(mut source: impl Source<'de, S>) -> IoResult<Value<S>> {
    expect_char(&mut source, b';')?;
    let len = parse_before::<'_, usize, _, _>(&mut source, b':')?;
    expect_char(&mut source, b'{')?;
    let mut vec = Vec::with_capacity(len);
    if len > source.limit() {
        return Err(Error::UnexpectedEof.into());
    }
    for _ in 0..len {
        let key = match Value::from_source(&mut source)? {
            Value::Int(int) => ArrayKey::Int(int),
            Value::String(string) => ArrayKey::String(string),
            _ => return Err(Error::BadArrayKeyType(source.offset()).into()),
        };
        let value = Value::from_source(&mut source)?;
        vec.push((key, value));
    }
    expect_char(&mut source, b'}')?;
    Ok(Value::Array(vec))
}

fn read_object<'de, S: Str<'de>>(mut source: impl Source<'de, S>) -> IoResult<Value<S>> {
    expect_char(&mut source, b':')?;
    let len = parse_before::<'_, usize, _, _>(&mut source, b':')?;
    expect_char(&mut source, b'"')?;
    let class = source.read_str(len)?;
    expect_char(&mut source, b'"')?;
    expect_char(&mut source, b':')?;
    let properties_len = parse_before::<'_, usize, _, _>(&mut source, b':')?;
    if properties_len > source.limit() {
        return Err(Error::UnexpectedEof.into());
    }
    expect_char(&mut source, b'{')?;

    let mut properties = Vec::with_capacity(properties_len);
    for _ in 0..properties_len {
        let name = match Value::from_source(&mut source)? {
            Value::String(string) => string,
            _ => return Err(Error::BadObjectKeyType(source.offset()).into()),
        };

        let name_bytes = name.as_bytes();
        let (name, vis) = if name_bytes.get(0) == Some(&0) {
            if name_bytes.get(1) == Some(&b'*') {
                if name_bytes.get(2) != Some(&0) {
                    return Err(Error::BadToken(source.offset()).into());
                }
                // encoding and length checked above
                (unsafe { name.range_from(3) }, PropertyVis::Protected)
            } else {
                let second_null = name_bytes
                    .iter()
                    .skip(1)
                    .position(|&b| b == 0)
                    .ok_or_else(|| Error::UnexpectedEof)?
                    + 1; // +1 because skip(1)
                let priv_class = unsafe { name.range(1, second_null) };
                (
                    unsafe { name.range_from(second_null + 1) },
                    PropertyVis::Private(priv_class),
                )
            }
        } else {
            (name, PropertyVis::Public)
        };

        let value = Value::from_source(&mut source)?;
        properties.push((PropertyName::new(vis, name), value));
    }

    expect_char(&mut source, b'}')?;

    Ok(Value::Object(Object::new(class, properties)))
}

fn read_ser<'de, S: Str<'de>>(mut source: impl Source<'de, S>) -> IoResult<Value<S>> {
    expect_char(&mut source, b':')?;
    let class_len = parse_before::<'_, usize, _, _>(&mut source, b':')?;
    expect_char(&mut source, b'"')?;
    let class = source.read_str(class_len)?;
    expect_char(&mut source, b'"')?;
    expect_char(&mut source, b':')?;

    let data_len = parse_before::<'_, usize, _, _>(&mut source, b':')?;
    expect_char(&mut source, b'{')?;
    let data = source.read_str(data_len)?;
    expect_char(&mut source, b'}')?;

    Ok(Value::Serializable(Serializable::new(class, data)))
}

fn read_ref<'de, S: Str<'de>>(mut source: impl Source<'de, S>) -> IoResult<Value<S>> {
    expect_char(&mut source, b':')?;
    let index = parse_before::<'_, usize, _, _>(&mut source, b';')?;

    Ok(Value::Reference(Ref::new(index)))
}

fn expect_char<'de, S: Str<'de>>(mut source: impl Source<'de, S>, char: u8) -> IoResult {
    if source.read_u8_char()? == char {
        Ok(())
    } else {
        Err(Error::BadToken(source.offset()).into())
    }
}

fn parse_before<'de, T: str::FromStr, S: Str<'de>, Src: Source<'de, S>>(
    mut source: Src,
    char: u8,
) -> IoResult<T> {
    let bytes = unsafe { source.read_until(char) }?;
    let str = str::from_utf8(bytes.as_bytes()).map_err(|_| Error::BadNumber(source.offset()))?;
    let ret = str
        .parse::<T>()
        .map_err(|_| Error::BadNumber(source.offset()))?;
    Ok(ret)
}
